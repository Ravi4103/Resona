use crate::library::cue;
use crate::library::db;
use crate::library::track::{extract_artwork_bytes, extract_lyrics_from_file, normalize_genre_name, split_genre_string, Genre, Track};
use crate::library::track_artwork::TrackArtworkCache;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use sqlx::SqlitePool;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use walkdir::WalkDir;

#[derive(Serialize)]
pub struct ScanResult {
    pub added: usize,
    pub updated: usize,
    pub removed: usize,
}

pub async fn scan_folder(pool: &SqlitePool, folder: &Path, artwork_cache: &TrackArtworkCache) -> Result<ScanResult, String> {
    let mut tx = pool.begin().await.map_err(|e| format!("Failed to begin transaction: {e}"))?;

    let mut added = 0;
    let updated = 0;
    let exts = ["mp3", "flac", "wav", "ogg", "m4a", "aac", "wma", "opus"];
    for entry in WalkDir::new(folder)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if exts.contains(&ext.to_lowercase().as_str()) {
                match Track::from_path(path) {
                    Ok(mut track) => {
                        if track.has_artwork && track.artwork_key.is_none() {
                            if let Some(bytes) = extract_artwork_bytes(path) {
                                if let Ok(key) = artwork_cache.save(&bytes) {
                                    track.artwork_key = Some(key);
                                }
                            }
                        }
                        let parent_id = track.id.clone();
                        db::upsert_track(&mut *tx, &track).await?;
                        added += 1;

                        // Process genres: split, normalize, upsert, link
                        if !track.raw_genre_names.is_empty() {
                            let genre_names = split_genre_string(&track.raw_genre_names);
                            let mut genre_ids: Vec<String> = Vec::new();
                            for name in &genre_names {
                                let nk = normalize_genre_name(name);
                                let gid = format!("genre-{}", nk);
                                let genre = Genre {
                                    id: gid.clone(),
                                    name: name.clone(),
                                    normalization_key: nk,
                                    track_count: 0,
                                };
                                let _ = db::upsert_genre(&mut *tx, &genre).await;
                                genre_ids.push(gid);
                            }
                            let _ = db::set_track_genres(&mut *tx, &track.id, &genre_ids).await;
                        }

                        if let Some((meta_lyrics, meta_source)) = extract_lyrics_from_file(path) {
                            let _ = db::upsert_lyrics(&mut *tx, &track.id, "", "", &meta_lyrics, &meta_source).await;
                        }

                        if let Some(cue_content) = cue::find_cue_for_audio(path) {
                            let cue_tracks = cue::parse_cue(&cue_content);
                            for ct in &cue_tracks {
                                let mut virtual_track = track.clone();
                                virtual_track.id = format!("{}-cue-{:02}", parent_id, ct.track_number);
                                virtual_track.title = ct.title.clone();
                                if !ct.artist.is_empty() {
                                    virtual_track.artist = ct.artist.clone();
                                }
                                virtual_track.track_number = ct.track_number;
                                virtual_track.cue_parent_id = Some(parent_id.clone());
                                virtual_track.cue_offset = ct.start_secs;
                                db::upsert_track(&mut *tx, &virtual_track).await?;
                                added += 1;
                                // Inherit parent's genres
                                if !virtual_track.raw_genre_names.is_empty() {
                                    let genre_names = split_genre_string(&virtual_track.raw_genre_names);
                                    let mut genre_ids: Vec<String> = Vec::new();
                                    for name in &genre_names {
                                        let nk = normalize_genre_name(name);
                                        let gid = format!("genre-{}", nk);
                                        let genre = Genre {
                                            id: gid.clone(),
                                            name: name.clone(),
                                            normalization_key: nk,
                                            track_count: 0,
                                        };
                                        let _ = db::upsert_genre(&mut *tx, &genre).await;
                                        genre_ids.push(gid);
                                    }
                                    let _ = db::set_track_genres(&mut *tx, &virtual_track.id, &genre_ids).await;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to read metadata for {:?}: {e}", path);
                    }
                }
            }
        }
    }

    // Propagate artwork_key to tracks in the same album that have none
    let _ = sqlx::query(
        "UPDATE tracks SET artwork_key = (
            SELECT t2.artwork_key FROM tracks t2
            WHERE t2.album = tracks.album
              AND t2.artwork_key IS NOT NULL AND t2.artwork_key != ''
            LIMIT 1
        ) WHERE artwork_key IS NULL AND album IN (
            SELECT album FROM tracks
            WHERE artwork_key IS NOT NULL AND artwork_key != '' AND album != ''
        )"
    )
    .execute(&mut *tx)
    .await;

    // Remove tracks whose files no longer exist on disk in this folder
    let sep = std::path::MAIN_SEPARATOR;
    let folder_str = folder.to_string_lossy();
    let with_sep = format!("{}{}", folder_str, sep);
    let escaped = with_sep.replace('%', "^%").replace('_', "^_");
    let like_pattern = format!("{}%", escaped);
    let orphan_ids: Vec<(String, String)> = sqlx::query_as(
        "SELECT id, file_path FROM tracks WHERE file_path = ? OR file_path LIKE ? ESCAPE '^'"
    )
    .bind(folder_str.as_ref())
    .bind(&like_pattern)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| format!("Failed to fetch orphan tracks: {e}"))?;

    let mut missing_ids: Vec<String> = Vec::new();
    for (id, file_path) in &orphan_ids {
        if !std::path::Path::new(file_path).exists() {
            missing_ids.push(id.clone());
        }
    }

    // Also find CUE children of missing tracks
    if !missing_ids.is_empty() {
        let params: Vec<String> = missing_ids.iter().enumerate().map(|(i, _)| format!("?{}", i + 1)).collect();
        let cue_sql = format!("SELECT id FROM tracks WHERE cue_parent_id IN ({})", params.join(","));
        let mut q = sqlx::query_scalar::<_, String>(&cue_sql);
        for id in &missing_ids {
            q = q.bind(id);
        }
        if let Ok(children) = q.fetch_all(&mut *tx).await {
            missing_ids.extend(children);
        }
    }

    let mut actually_removed = 0;
    if !missing_ids.is_empty() {
        let params: Vec<String> = missing_ids.iter().enumerate().map(|(i, _)| format!("?{}", i + 1)).collect();
        let placeholders = params.join(",");

        let del_hist = format!("DELETE FROM play_history WHERE track_id IN ({})", placeholders);
        let mut q = sqlx::query(&del_hist);
        for id in &missing_ids { q = q.bind(id); }
        let _ = q.execute(&mut *tx).await;

        let del_fav = format!("DELETE FROM favorites WHERE track_id IN ({})", placeholders);
        let mut q = sqlx::query(&del_fav);
        for id in &missing_ids { q = q.bind(id); }
        let _ = q.execute(&mut *tx).await;

        let del_pl = format!("DELETE FROM playlist_tracks WHERE track_id IN ({})", placeholders);
        let mut q = sqlx::query(&del_pl);
        for id in &missing_ids { q = q.bind(id); }
        let _ = q.execute(&mut *tx).await;

        let del_trk = format!("DELETE FROM tracks WHERE id IN ({})", placeholders);
        let mut q = sqlx::query(&del_trk);
        for id in &missing_ids { q = q.bind(id); }
        q.execute(&mut *tx).await.map_err(|e| format!("Failed to delete orphan tracks: {e}"))?;

        actually_removed = missing_ids.len();
    }

    tx.commit().await.map_err(|e| format!("Failed to commit transaction: {e}"))?;

    Ok(ScanResult {
        added,
        updated,
        removed: actually_removed,
    })
}

pub fn start_watcher(
    pool: SqlitePool,
    folder: std::path::PathBuf,
) -> Result<impl Watcher, String> {
    let pool = Arc::new(Mutex::new(pool));
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let pool = Arc::clone(&pool);
                let path = event.paths.first().cloned();
                if let Some(path) = path {
                    let exts = ["mp3", "flac", "wav", "ogg", "m4a", "aac", "wma", "opus"];
                    let is_audio = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|e| exts.contains(&e.to_lowercase().as_str()))
                        .unwrap_or(false);
                    if !is_audio {
                        return;
                    }
                    tokio::spawn(async move {
                        let pool = pool.lock().await;
                        match event.kind {
                            EventKind::Create(_) => {
                                if let Ok(track) = Track::from_path(&path) {
                                    let _ = db::upsert_track(&*pool, &track).await;
                                }
                            }
                            EventKind::Remove(_) => {
                                let path_str = path.to_string_lossy().to_string();
                                // Try to find track by path and delete
                                if let Ok(Some(t)) = db::get_track_by_path(&pool, &path_str).await {
                                    let _ = db::delete_track(&pool, &t.id).await;
                                }
                            }
                            _ => {}
                        }
                    });
                }
            }
        },
        Config::default(),
    )
    .map_err(|e| format!("Failed to create watcher: {e}"))?;
    watcher
        .watch(&folder, RecursiveMode::Recursive)
        .map_err(|e| format!("Failed to watch folder: {e}"))?;
    Ok(watcher)
}
