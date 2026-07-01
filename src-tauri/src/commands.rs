use crate::library::db;
use crate::library::palette;
use crate::library::playlist;
use crate::library::scanner;
use crate::library::track::Track;
use crate::state::AppState;
use lofty::file::TaggedFileExt;
use std::path::Path;
use std::fs;
use tauri::Emitter;
use tauri::State;
use serde::{Deserialize, Serialize};


#[tauri::command]
pub async fn get_tracks(state: State<'_, AppState>) -> Result<Vec<Track>, String> {
    db::get_all_tracks(&state.db).await
}

#[tauri::command]
pub async fn get_track(state: State<'_, AppState>, id: String) -> Result<Option<Track>, String> {
    db::get_track(&state.db, &id).await
}

#[tauri::command]
pub async fn search_tracks(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<Track>, String> {
    let track_ids = {
        let search = state.search.lock().map_err(|e| e.to_string())?;
        search.search(&query, 50)
    };
    let mut results = Vec::new();
    for id in track_ids {
        if let Ok(Some(t)) = db::get_track(&state.db, &id).await {
            results.push(t);
        }
    }
    Ok(results)
}

#[tauri::command]
pub async fn get_all_genres(state: State<'_, AppState>) -> Result<Vec<crate::library::track::Genre>, String> {
    db::get_all_genres(&state.db).await
}

#[tauri::command]
pub async fn get_tracks_by_genre_id(state: State<'_, AppState>, genre_id: String) -> Result<Vec<Track>, String> {
    db::get_tracks_by_genre_id(&state.db, &genre_id).await
}

#[tauri::command]
pub async fn scan_folder(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<scanner::ScanResult, String> {
    if state.is_scanning.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return Ok(scanner::ScanResult { added: 0, updated: 0, removed: 0 });
    }
    let scan_result = scanner::scan_folder(&state.db, Path::new(&path), &state.track_artwork_cache).await;
    state.is_scanning.store(false, std::sync::atomic::Ordering::SeqCst);
    let result = scan_result?;
    let tracks = db::get_all_tracks(&state.db).await?;
    let mut search = state.search.lock().map_err(|e| e.to_string())?;
    search.rebuild(&tracks);
    drop(search);
    if let Ok(mut cache) = state.artwork_cache.lock() {
        cache.clear();
    }
    let _ = app_handle.emit("library:sync-finished", &result);
    Ok(result)
}

fn extract_artwork_base64(file_path: &str) -> Result<String, String> {
    let tagged_file = lofty::read_from_path(Path::new(file_path))
        .map_err(|e| format!("Failed to read file: {e}"))?;
    let picture = tagged_file
        .tags()
        .iter()
        .filter_map(|tag| tag.pictures().first())
        .next()
        .ok_or("No artwork")?;
    use base64::Engine;
    Ok(base64::engine::general_purpose::STANDARD.encode(picture.data()))
}

#[tauri::command]
pub async fn get_artwork(state: State<'_, AppState>, id: String) -> Result<String, String> {
    {
        let cache = state.artwork_cache.lock().map_err(|e| e.to_string())?;
        if let Some(cached) = cache.get(&id) {
            return Ok(cached.clone());
        }
    }
    let track = db::get_track(&state.db, &id)
        .await?
        .ok_or("Track not found")?;
    let artwork = extract_artwork_base64(&track.file_path)?;
    let mut cache = state.artwork_cache.lock().map_err(|e| e.to_string())?;
    cache.insert(id, artwork.clone());
    Ok(artwork)
}

fn invalidate_artwork_cache(state: &AppState, ids: &[String]) {
    if let Ok(mut cache) = state.artwork_cache.lock() {
        for id in ids {
            cache.remove(id);
        }
    }
}

#[tauri::command]
pub async fn set_artwork(app_handle: tauri::AppHandle, state: State<'_, AppState>, id: String, image_b64: String) -> Result<(), String> {
    let track = db::get_track(&state.db, &id)
        .await?
        .ok_or("Track not found")?;
    let path = std::path::Path::new(&track.file_path);

    use base64::Engine;
    let image_data = base64::engine::general_purpose::STANDARD
        .decode(&image_b64)
        .map_err(|e| format!("Invalid base64: {e}"))?;

    use lofty::config::{ParseOptions, WriteOptions};
    use lofty::file::{BoundTaggedFile, TaggedFileExt};
    use lofty::picture::{Picture, PictureType, MimeType};
    use std::fs::OpenOptions;

    let file = OpenOptions::new().read(true).write(true).open(path)
        .map_err(|e| format!("Failed to open file: {e}"))?;
    let mut bound = BoundTaggedFile::read_from(file, ParseOptions::new())
        .map_err(|e| format!("Failed to read tags: {e}"))?;

    // Remove existing front cover pictures
    if let Some(tag) = bound.primary_tag_mut() {
        tag.remove_picture_type(PictureType::CoverFront);
    }

    // Add new picture
    let picture = Picture::new_unchecked(
        PictureType::CoverFront,
        Some(MimeType::Jpeg),
        None,
        image_data,
    );
    if let Some(tag) = bound.primary_tag_mut() {
        tag.push_picture(picture);
    }

    // Save back to file
    bound.save(WriteOptions::default())
        .map_err(|e| format!("Failed to write artwork: {e}"))?;

    // Update DB flag
    sqlx::query("UPDATE tracks SET has_artwork = 1 WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| format!("DB error: {e}"))?;

    invalidate_artwork_cache(&state, &[id.clone()]);
                let _ = app_handle.emit("library:track-updated", &id);
    Ok(())
}

#[tauri::command]
pub async fn get_track_artwork(
    state: State<'_, AppState>,
    id: String,
    size: Option<String>,
) -> Result<Option<Vec<u8>>, String> {
    let track = db::get_track(&state.db, &id)
        .await?
        .ok_or("Track not found")?;
    let variant = size.as_deref();
    if let Some(ref k) = track.artwork_key {
        if state.track_artwork_cache.exists(k, variant) {
            return state.track_artwork_cache.read(k, variant).map(Some);
        }
        if variant.is_some() {
            if let Ok(data) = state.track_artwork_cache.read(k, None) {
                return Ok(Some(data));
            }
        }
    }
    // Fallback: extract artwork directly from the audio file and cache it
    if let Some(bytes) = crate::library::track::extract_artwork_bytes(Path::new(&track.file_path)) {
        match state.track_artwork_cache.save(&bytes) {
            Ok(new_key) => {
                let _ = sqlx::query("UPDATE tracks SET artwork_key = ?, has_artwork = 1 WHERE id = ?")
                    .bind(&new_key)
                    .bind(&id)
                    .execute(&state.db)
                    .await;
                if state.track_artwork_cache.exists(&new_key, variant) {
                    return state.track_artwork_cache.read(&new_key, variant).map(Some);
                }
                return state.track_artwork_cache.read(&new_key, None).map(Some);
            }
            Err(_) => return Ok(None),
        }
    }
    Ok(None)
}

async fn get_artwork_bytes(state: &AppState, track: &Track) -> Option<Vec<u8>> {
    if let Some(ref k) = track.artwork_key {
        if let Ok(data) = state.track_artwork_cache.read(k, None) {
            return Some(data);
        }
    }
    let bytes = crate::library::track::extract_artwork_bytes(std::path::Path::new(&track.file_path))?;
    match state.track_artwork_cache.save(&bytes) {
        Ok(new_key) => {
            let _ = sqlx::query("UPDATE tracks SET artwork_key = ?, has_artwork = 1 WHERE id = ?")
                .bind(&new_key)
                .bind(&track.id)
                .execute(&state.db)
                .await;
            Some(bytes)
        }
        Err(_) => None,
    }
}

#[tauri::command]
pub async fn get_track_palette(state: State<'_, AppState>, id: String) -> Result<Option<palette::ThemeColors>, String> {
    let track = db::get_track(&state.db, &id)
        .await?
        .ok_or("Track not found")?;
    let bytes = get_artwork_bytes(&state, &track).await;
    match bytes {
        Some(b) => Ok(palette::extract_palette(&b)),
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn play_track(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    id: String,
) -> Result<(), String> {
    let track = db::get_track(&state.db, &id)
        .await?
        .ok_or("Track not found")?;

    // Determine the actual file to play (parent file for CUE tracks)
    let play_path = if let Some(parent_id) = &track.cue_parent_id {
        let parent = db::get_track(&state.db, parent_id)
            .await?
            .ok_or("CUE parent track not found")?;
        parent.file_path.clone()
    } else {
        track.file_path.clone()
    };

    if !std::path::Path::new(&play_path).exists() {
        return Err(format!("File not found: {play_path}"));
    }

    state.player.play_track(std::path::PathBuf::from(&play_path), app_handle.clone())?;

    // If this is a CUE virtual track, seek to the track offset
    if track.cue_offset > 0.0 {
        state.player.seek(track.cue_offset as f32);
    }

    // Record the play
    let _ = db::record_play(&state.db, &id).await;
    // Store current track ID for cross-window state sync
    if let Ok(mut tid) = state.current_track_id.lock() {
        *tid = Some(id.clone());
    }
    // Emit event so frontend knows a new track started
    let _ = app_handle.emit("track-started", &id);
    let _ = app_handle.emit("player:status", serde_json::json!({"playing": true, "track_id": id}));
    Ok(())
}

#[tauri::command]
pub async fn record_play(state: State<'_, AppState>, id: String) -> Result<(), String> {
    db::record_play(&state.db, &id).await
}

#[tauri::command]
pub async fn get_recently_played_tracks(
    state: State<'_, AppState>,
    limit: i64,
) -> Result<Vec<Track>, String> {
    db::get_recently_played_tracks(&state.db, limit).await
}

#[tauri::command]
pub async fn get_most_listened_tracks(
    state: State<'_, AppState>,
    limit: i64,
) -> Result<Vec<Track>, String> {
    db::get_most_listened_tracks(&state.db, limit).await
}

#[tauri::command]
pub async fn get_least_listened_tracks(
    state: State<'_, AppState>,
    limit: i64,
) -> Result<Vec<Track>, String> {
    db::get_least_listened_tracks(&state.db, limit).await
}

#[tauri::command]
pub async fn toggle_play(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let result = state.player.toggle();
    let _ = app_handle.emit("player:status", serde_json::json!({"paused": result}));
    Ok(result)
}

#[tauri::command]
pub async fn seek(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    pos_secs: f64,
) -> Result<(), String> {
    state.player.seek(pos_secs as f32);
    let _ = app_handle.emit("player:status", serde_json::json!({"position": pos_secs}));
    Ok(())
}

#[tauri::command]
pub async fn set_volume(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    vol: f32,
) -> Result<(), String> {
    state.player.set_volume(vol);
    let _ = app_handle.emit("player:status", serde_json::json!({"volume": vol}));
    Ok(())
}

#[tauri::command]
pub async fn set_eq_band(
    state: State<'_, AppState>,
    index: u8,
    gain_db: f32,
) -> Result<(), String> {
    if index >= 10 {
        return Err("EQ band index must be 0-9".into());
    }
    state.player.set_eq_band(index, gain_db.clamp(-12.0, 12.0));
    Ok(())
}

#[tauri::command]
pub async fn set_eq_enabled(
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<(), String> {
    state.player.set_eq_enabled(enabled);
    Ok(())
}

#[derive(Serialize)]
pub struct PlayerStateResponse {
    pub track_id: Option<String>,
    pub is_playing: bool,
    pub position: f64,
    pub volume: f64,
}

#[tauri::command]
pub async fn get_position(state: State<'_, AppState>) -> Result<f64, String> {
    Ok(state.player.position() as f64)
}

#[tauri::command]
pub async fn get_player_state(state: State<'_, AppState>) -> Result<PlayerStateResponse, String> {
    let track_id = state.current_track_id.lock().map_err(|e| e.to_string())?.clone();
    Ok(PlayerStateResponse {
        track_id,
        is_playing: state.player.is_playing(),
        position: state.player.position() as f64,
        volume: f32::from_bits(state.volume.load(std::sync::atomic::Ordering::Acquire)) as f64,
    })
}

#[tauri::command]
pub async fn get_playlists(
    state: State<'_, AppState>,
) -> Result<Vec<playlist::Playlist>, String> {
    playlist::get_playlists(&state.db).await
}

#[tauri::command]
pub async fn create_playlist(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    name: String,
) -> Result<playlist::Playlist, String> {
    let p = playlist::create_playlist(&state.db, &name).await?;
    let _ = app_handle.emit("playlist:created", &p);
    Ok(p)
}

#[tauri::command]
pub async fn delete_playlist(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    playlist::delete_playlist(&state.db, &id).await?;
    let _ = app_handle.emit("playlist:deleted", &id);
    Ok(())
}

#[tauri::command]
pub async fn rename_playlist(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> Result<(), String> {
    playlist::rename_playlist(&state.db, &id, &name).await?;
    let _ = app_handle.emit("playlist:created", &serde_json::json!({"id": id, "name": name}));
    Ok(())
}

#[tauri::command]
pub async fn export_playlist_m3u(
    state: State<'_, AppState>,
    id: String,
    output_path: String,
) -> Result<(), String> {
    playlist::export_playlist_m3u(&state.db, &id, &output_path).await
}

#[tauri::command]
pub async fn import_playlist_m3u(
    state: State<'_, AppState>,
    file_path: String,
    name: String,
) -> Result<playlist::Playlist, String> {
    playlist::import_playlist_m3u(&state.db, &file_path, &name).await
}

#[tauri::command]
pub async fn get_playlist_track_count(
    state: State<'_, AppState>,
    id: String,
) -> Result<i32, String> {
    let count: (i32,) = sqlx::query_as("SELECT COUNT(*) FROM playlist_tracks WHERE playlist_id = ?")
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| format!("{e}"))?;
    Ok(count.0)
}

#[tauri::command]
pub async fn reorder_playlist_tracks(
    state: State<'_, AppState>,
    playlist_id: String,
    track_ids: Vec<String>,
) -> Result<(), String> {
    let mut tx = state.db.begin().await.map_err(|e| format!("{e}"))?;
    for (i, track_id) in track_ids.iter().enumerate() {
        sqlx::query("UPDATE playlist_tracks SET position = ? WHERE playlist_id = ? AND track_id = ?")
            .bind(i as i32)
            .bind(&playlist_id)
            .bind(track_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("Failed to reorder: {e}"))?;
    }
    tx.commit().await.map_err(|e| format!("{e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn add_to_playlist(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    playlist_id: String,
    track_id: String,
) -> Result<(), String> {
    playlist::add_track_to_playlist(&state.db, &playlist_id, &track_id).await?;
    let _ = app_handle.emit("playlist:tracks-changed", &playlist_id);
    Ok(())
}

#[tauri::command]
pub async fn add_tracks_to_playlist(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    playlist_id: String,
    track_ids: Vec<String>,
) -> Result<(), String> {
    for id in &track_ids {
        playlist::add_track_to_playlist(&state.db, &playlist_id, id).await?;
    }
    let _ = app_handle.emit("playlist:tracks-changed", &playlist_id);
    Ok(())
}

#[tauri::command]
pub async fn remove_from_playlist(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    playlist_id: String,
    track_id: String,
) -> Result<(), String> {
    playlist::remove_track_from_playlist(&state.db, &playlist_id, &track_id).await?;
    let _ = app_handle.emit("playlist:tracks-changed", &playlist_id);
    Ok(())
}

#[tauri::command]
pub async fn get_playlist_tracks(
    state: State<'_, AppState>,
    playlist_id: String,
) -> Result<Vec<Track>, String> {
    playlist::get_playlist_tracks(&state.db, &playlist_id).await
}

#[tauri::command]
pub async fn get_favorite_ids(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    db::get_favorite_ids(&state.db).await
}

#[tauri::command]
pub async fn toggle_favorite(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<bool, String> {
    let result = db::toggle_favorite(&state.db, &id).await?;
    let _ = app_handle.emit("library:track-updated", &id);
    Ok(result)
}

#[tauri::command]
pub async fn get_library_folders(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    db::get_library_folders(&state.db).await
}

#[tauri::command]
pub async fn add_library_folder(state: State<'_, AppState>, path: String) -> Result<(), String> {
    db::add_library_folder(&state.db, &path).await
}

#[tauri::command]
pub async fn add_library_folders(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> Result<scanner::ScanResult, String> {
    if paths.is_empty() {
        return Ok(scanner::ScanResult { added: 0, updated: 0, removed: 0 });
    }
    if state.is_scanning.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return Err("A scan is already in progress".into());
    }
    let mut total_added: usize = 0;
    let mut total_removed: usize = 0;
    for path in &paths {
        let _ = app_handle.emit("folder-scan-progress", serde_json::json!({
            "path": path,
            "status": "scanning",
        }));
        let _ = db::add_library_folder(&state.db, path).await;
        match scanner::scan_folder(&state.db, Path::new(path), &state.track_artwork_cache).await {
            Ok(r) => {
                total_added += r.added;
                total_removed += r.removed;
                let _ = app_handle.emit("folder-scan-progress", serde_json::json!({
                    "path": path,
                    "status": "done",
                    "added": r.added,
                    "updated": r.updated,
                    "removed": r.removed,
                }));
            }
            Err(e) => {
                let _ = app_handle.emit("folder-scan-progress", serde_json::json!({
                    "path": path,
                    "status": "error",
                    "error": e,
                }));
            }
        }
    }
    state.is_scanning.store(false, std::sync::atomic::Ordering::SeqCst);
    let result = scanner::ScanResult { added: total_added, updated: 0, removed: total_removed };
    let tracks = db::get_all_tracks(&state.db).await?;
    let mut search = state.search.lock().map_err(|e| e.to_string())?;
    search.rebuild(&tracks);
    drop(search);
    if let Ok(mut cache) = state.artwork_cache.lock() {
        cache.clear();
    }
    let _ = app_handle.emit("library:sync-finished", &result);
    Ok(result)
}

#[tauri::command]
pub async fn remove_library_folder(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<(), String> {
    db::remove_library_folder(&state.db, &path).await?;
    let tracks = db::get_all_tracks(&state.db).await?;
    let mut search = state.search.lock().map_err(|e| e.to_string())?;
    search.rebuild(&tracks);
    let _ = app_handle.emit("library:sync-finished", serde_json::json!({"removed_folder": path}));
    Ok(())
}

#[tauri::command]
pub async fn delete_tracks(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    ids: Vec<String>,
    delete_files: bool,
) -> Result<(), String> {
    let file_paths = db::delete_tracks(&state.db, &ids).await?;
    if delete_files {
        for path in &file_paths {
            let _ = fs::remove_file(path);
        }
    }
    let mut search = state.search.lock().map_err(|e| e.to_string())?;
    for id in &ids {
        search.remove_track(id);
    }
    let _ = app_handle.emit("library:track-deleted", &ids);
    Ok(())
}

#[tauri::command]
pub async fn get_lyrics(file_path: String) -> Result<Option<String>, String> {
    let p = Path::new(&file_path);
    let base = p.with_extension("");
    for ext in &["lrc", "txt"] {
        let lyric_path = base.with_extension(ext);
        if lyric_path.exists() {
            return fs::read_to_string(&lyric_path)
                .map(Some)
                .map_err(|e| format!("Failed to read lyrics: {e}"));
        }
    }
    Ok(None)
}

#[tauri::command]
pub async fn show_in_explorer(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let track = db::get_track(&state.db, &id)
        .await?
        .ok_or("Track not found")?;
    let path = std::path::PathBuf::from(&track.file_path);
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg("/select,")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open explorer: {e}"))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open finder: {e}"))?;
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(parent) = path.parent() {
            std::process::Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .map_err(|e| format!("Failed to open file manager: {e}"))?;
        }
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LyricsResult {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub lyrics: String,
    pub source: String,
}

#[tauri::command]
pub async fn find_lyrics(
    state: State<'_, AppState>,
    title: String,
    artist: String,
    album: String,
    save: Option<String>,
) -> Result<Vec<LyricsResult>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let mut results = Vec::new();

    // Try LRCLIB
    let lrclib_url = format!(
        "https://lrclib.net/api/get?artist_name={}&track_name={}&album_name={}",
        urlencoding(&artist),
        urlencoding(&title),
        urlencoding(&album),
    );
    if let Ok(resp) = client.get(&lrclib_url).send().await {
        if resp.status().is_success() {
            if let Ok(data) = resp.json::<serde_json::Value>().await {
                if let Some(lyrics) = data["syncedLyrics"].as_str().filter(|l| !l.is_empty())
                    .or_else(|| data["plainLyrics"].as_str().filter(|l| !l.is_empty()))
                {
                    results.push(LyricsResult {
                        title: data["trackName"].as_str().unwrap_or(&title).to_string(),
                        artist: data["artistName"].as_str().unwrap_or(&artist).to_string(),
                        album: data["albumName"].as_str().unwrap_or(&album).to_string(),
                        lyrics: lyrics.to_string(),
                        source: "LRCLIB".to_string(),
                    });
                }
            }
        }
    }

    // Try KuGou (if LRCLIB returned nothing)
    if results.is_empty() {
        let search_url = format!(
            "https://lyrics.kugou.com/search?ver=1&man=yes&client=pc&keyword={}%20{}&duration=&hash=",
            urlencoding(&title),
            urlencoding(&artist),
        );
        if let Ok(resp) = client.get(&search_url).send().await {
            if resp.status().is_success() {
                if let Ok(data) = resp.json::<serde_json::Value>().await {
                    if let Some(candidates) = data["candidates"].as_array() {
                        if let Some(first) = candidates.first() {
                            if let (Some(id), Some(access_token)) = (
                                first["id"].as_str(),
                                first["access_token"].as_str().or(Some("")),
                            ) {
                                let dl_url = format!(
                                    "https://lyrics.kugou.com/download?ver=1&client=pc&id={id}&accessToken={access_token}&fmt=lrc"
                                );
                                if let Ok(dl_resp) = client.get(&dl_url).send().await {
                                    if dl_resp.status().is_success() {
                                        if let Ok(dl_data) = dl_resp.json::<serde_json::Value>().await {
                                            if let Some(content) = dl_data["content"].as_str() {
                                                if let Ok(decoded) = base64_decode(content) {
                                                    if let Ok(lyrics) = String::from_utf8(decoded) {
                                                        let lrc_text = if lyrics.trim().starts_with("[ti:")
                                                            || lyrics.trim().contains("[00:")
                                                        {
                                                            lyrics
                                                        } else {
                                                            // If not LRC format, wrap as plain text
                                                            lyrics
                                                        };
                                                        results.push(LyricsResult {
                                                            title: first["song_name"].as_str().unwrap_or(&title).to_string(),
                                                            artist: first["singer_name"].as_str().unwrap_or(&artist).to_string(),
                                                            album: String::new(),
                                                            lyrics: lrc_text,
                                                            source: "KuGou".to_string(),
                                                        });
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some(ref track_id) = save {
        if let Some(first) = results.first() {
            let source = if first.lyrics.contains('[') && first.lyrics.contains(']') {
                "provider-synced"
            } else {
                "provider-plain"
            };
            let _ = db::upsert_lyrics(&state.db, track_id, &first.lyrics, source, "", "").await;
        }
    }

    Ok(results)
}

#[tauri::command]
pub async fn save_lyrics(
    state: State<'_, AppState>,
    track_id: String,
    content: String,
    source: String,
) -> Result<(), String> {
    db::upsert_lyrics(&state.db, &track_id, &content, &source, "", "").await
}

#[tauri::command]
pub async fn get_track_lyrics(
    state: State<'_, AppState>,
    track_id: String,
) -> Result<Option<db::LyricRow>, String> {
    db::get_lyrics(&state.db, &track_id).await
}

#[tauri::command]
pub async fn batch_fetch_lyrics(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    folder_path: Option<String>,
) -> Result<String, String> {
    if state.is_fetching.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return Ok("already_running".to_string());
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let db = state.db.clone();
    let guard = state.is_fetching.clone();

    tauri::async_runtime::spawn(async move {
        let tracks = match db::get_all_tracks(&db).await {
            Ok(t) => t,
            Err(e) => {
                log::error!("Failed to get tracks in background fetch: {e}");
                let _ = app_handle.emit("batch-lyrics-all-done", serde_json::json!({"error": e}));
                guard.store(false, std::sync::atomic::Ordering::SeqCst);
                return;
            }
        };

        let folders: Vec<String> = if let Some(ref fp) = folder_path {
            vec![fp.clone()]
        } else {
            match db::get_library_folders(&db).await {
                Ok(f) => f,
                Err(e) => {
                    log::error!("Failed to get folders in background fetch: {e}");
                    let _ = app_handle.emit("batch-lyrics-all-done", serde_json::json!({"error": e}));
                    guard.store(false, std::sync::atomic::Ordering::SeqCst);
                    return;
                }
            }
        };

        for folder in &folders {
            let folder_tracks: Vec<_> = tracks.iter().filter(|t| t.file_path.starts_with(folder)).collect();
            let total = folder_tracks.len();
            if total == 0 {
                continue;
            }

            let mut fetched = 0u32;
            let mut skipped_already_have = 0u32;
            let mut skipped_not_found = 0u32;

            for (i, track) in folder_tracks.iter().enumerate() {
                let existing = match db::get_lyrics(&db, &track.id).await {
                    Ok(e) => e,
                    Err(_) => continue,
                };
                let has_content = existing.as_ref().map_or(false, |r| {
                    !r.content.is_empty() || !r.meta_content.is_empty()
                });
                let is_not_found = existing.as_ref().map_or(false, |r| r.source == "not-found");

                if has_content {
                    skipped_already_have += 1;
                    continue;
                }
                if is_not_found {
                    skipped_not_found += 1;
                    continue;
                }

                let p = std::path::Path::new(&track.file_path);
                let base = p.with_extension("");
                let mut sibling_found = false;
                for ext in &["lrc", "txt"] {
                    let lyric_path = base.with_extension(ext);
                    if lyric_path.exists() {
                        if let Ok(text) = std::fs::read_to_string(&lyric_path) {
                            if !text.trim().is_empty() {
                                let _ = db::upsert_lyrics(&db, &track.id, "", "", &text, "sibling-file").await;
                                sibling_found = true;
                                skipped_already_have += 1;
                                break;
                            }
                        }
                    }
                }
                if sibling_found {
                    continue;
                }

                app_handle.emit("batch-lyrics-progress", serde_json::json!({
                    "current": i + 1,
                    "total": total,
                    "track": track.title,
                    "status": "searching",
                    "folder": folder,
                })).ok();

                let mut found: Option<(String, String)> = None;

                let lrclib_url = format!(
                    "https://lrclib.net/api/get?artist_name={}&track_name={}&album_name={}",
                    urlencoding(&track.artist),
                    urlencoding(&track.title),
                    urlencoding(&track.album),
                );
                if let Ok(resp) = client.get(&lrclib_url).send().await {
                    if resp.status().is_success() {
                        if let Ok(data) = resp.json::<serde_json::Value>().await {
                            if let Some(lyrics) = data["syncedLyrics"].as_str()
                                .or_else(|| data["plainLyrics"].as_str())
                                .filter(|l| !l.is_empty())
                            {
                                let source = if lyrics.contains('[') && lyrics.contains(']') {
                                    "lrclib-synced"
                                } else {
                                    "lrclib-plain"
                                };
                                found = Some((lyrics.to_string(), source.to_string()));
                            }
                        }
                    }
                }

                if found.is_none() {
                    let search_url = format!(
                        "https://lyrics.kugou.com/search?ver=1&man=yes&client=pc&keyword={}%20{}&duration=&hash=",
                        urlencoding(&track.title),
                        urlencoding(&track.artist),
                    );
                    if let Ok(resp) = client.get(&search_url).send().await {
                        if resp.status().is_success() {
                            if let Ok(data) = resp.json::<serde_json::Value>().await {
                                if let Some(candidates) = data["candidates"].as_array() {
                                    if let Some(first) = candidates.first() {
                                        if let (Some(id), Some(access_token)) = (
                                            first["id"].as_str(),
                                            first["access_token"].as_str().or(Some("")),
                                        ) {
                                            let dl_url = format!(
                                                "https://lyrics.kugou.com/download?ver=1&client=pc&id={id}&accessToken={access_token}&fmt=lrc"
                                            );
                                            if let Ok(dl_resp) = client.get(&dl_url).send().await {
                                                if dl_resp.status().is_success() {
                                                    if let Ok(dl_data) = dl_resp.json::<serde_json::Value>().await {
                                                        if let Some(content) = dl_data["content"].as_str() {
                                                            if let Ok(decoded) = base64_decode(content) {
                                                                if let Ok(lyrics) = String::from_utf8(decoded) {
                                                                    let source = if lyrics.contains('[') && lyrics.contains(']') {
                                                                        "kugou-synced"
                                                                    } else {
                                                                        "kugou-plain"
                                                                    };
                                                                    found = Some((lyrics, source.to_string()));
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                let was_found = found.is_some();
                if let Some((lyrics, source)) = found {
                    let _ = db::upsert_lyrics(&db, &track.id, &lyrics, &source, "", "").await;
                    fetched += 1;
                } else {
                    let _ = db::upsert_lyrics(&db, &track.id, "", "not-found", "", "").await;
                    skipped_not_found += 1;
                }

                app_handle.emit("batch-lyrics-progress", serde_json::json!({
                    "current": i + 1,
                    "total": total,
                    "track": track.title,
                    "status": if was_found { "found" } else { "not_found" },
                    "folder": folder,
                })).ok();

                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }

            app_handle.emit("batch-lyrics-folder-done", serde_json::json!({
                "folder": folder,
                "fetched": fetched,
                "skipped_already_have": skipped_already_have,
                "skipped_not_found": skipped_not_found,
                "total": total,
            })).ok();
        }

        app_handle.emit("batch-lyrics-all-done", serde_json::json!({})).ok();
        guard.store(false, std::sync::atomic::Ordering::SeqCst);
    });

    Ok("started".to_string())
}

fn base64_decode(s: &str) -> Result<Vec<u8>, String> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(s)
        .map_err(|e| format!("Base64 decode error: {e}"))
}

fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "%20".to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

#[derive(Debug, Deserialize)]
pub struct UpdateMetadataInput {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: String,
    pub genre: String,
    pub year: i32,
    pub track_number: u32,
    pub disc_number: u32,
}

#[tauri::command]
pub async fn update_track_metadata(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    input: UpdateMetadataInput,
) -> Result<(), String> {
    let track = db::get_track(&state.db, &input.id)
        .await?
        .ok_or("Track not found")?;

    // Update database
    let mut t = track.clone();
    t.title = input.title;
    t.artist = input.artist;
    t.album = input.album;
    t.album_artist = input.album_artist;
    t.genre = input.genre.clone();
    t.raw_genre_names = input.genre;
    t.year = input.year;
    t.track_number = input.track_number;
    t.disc_number = input.disc_number;
    db::upsert_track(&state.db, &t).await?;

    // Re-process genres
    use crate::library::track::{normalize_genre_name, split_genre_string, Genre as GenreModel};
    if !t.raw_genre_names.is_empty() {
        let genre_names = split_genre_string(&t.raw_genre_names);
        let mut genre_ids: Vec<String> = Vec::new();
        for name in &genre_names {
            let nk = normalize_genre_name(name);
            let gid = format!("genre-{}", nk);
            let genre = GenreModel {
                id: gid.clone(),
                name: name.clone(),
                normalization_key: nk,
                track_count: 0,
            };
            let _ = db::upsert_genre(&mut *state.db.acquire().await.map_err(|e| e.to_string())?, &genre).await;
            genre_ids.push(gid);
        }
        let _ = db::set_track_genres(&mut *state.db.acquire().await.map_err(|e| e.to_string())?, &t.id, &genre_ids).await;
    }

    // Update search index incrementally
    let mut search = state.search.lock().map_err(|e| e.to_string())?;
    search.index_track(&t);

    let _ = app_handle.emit("library:track-updated", &input.id);
    Ok(())
}

#[derive(Serialize)]
pub struct SearchAllResults {
    pub tracks: Vec<Track>,
    pub albums: Vec<SearchAlbumResult>,
    pub artists: Vec<String>,
    pub genres: Vec<String>,
    pub composers: Vec<String>,
}

#[derive(Serialize)]
pub struct SearchAlbumResult {
    pub name: String,
    pub artist: String,
    pub year: i32,
    pub track_count: i64,
    pub first_track_id: String,
}

#[tauri::command]
pub async fn search_all(
    state: State<'_, AppState>,
    query: String,
) -> Result<SearchAllResults, String> {
    let q = format!("%{}%", query.to_lowercase());

    // Search tracks (use existing search index)
    let track_ids = {
        let search = state.search.lock().map_err(|e| e.to_string())?;
        search.search(&query, 50)
    };
    let mut tracks = Vec::new();
    for id in track_ids {
        if let Ok(Some(t)) = db::get_track(&state.db, &id).await {
            tracks.push(t);
        }
    }

    // Search albums
    let albums = sqlx::query_as::<_, (String, String, i32, i64, String)>(
        "SELECT album, COALESCE(album_artist, artist, ''), MAX(year) as year, COUNT(*) as track_count, MIN(id) as first_track_id
         FROM tracks WHERE LOWER(album) LIKE ?1 OR LOWER(album_artist) LIKE ?1 OR LOWER(artist) LIKE ?1
         GROUP BY album ORDER BY album"
    )
    .bind(&q)
    .fetch_all(&state.db)
    .await
    .map_err(|e| format!("{e}"))?
    .into_iter()
    .map(|(name, artist, year, count, first_id)| SearchAlbumResult {
        name, artist, year, track_count: count, first_track_id: first_id,
    })
    .collect::<Vec<_>>();

    // Search artists
    let artists: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT artist FROM tracks WHERE LOWER(artist) LIKE ?1 AND artist != '' ORDER BY artist LIMIT 20"
    )
    .bind(&q)
    .fetch_all(&state.db)
    .await
    .map_err(|e| format!("{e}"))?;

    // Search genres
    let genres: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM genres WHERE LOWER(name) LIKE ?1 ORDER BY name LIMIT 20"
    )
    .bind(&q)
    .fetch_all(&state.db)
    .await
    .map_err(|e| format!("{e}"))?;

    // Search composers
    let composers: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT composer FROM tracks WHERE LOWER(composer) LIKE ?1 AND composer != '' ORDER BY composer LIMIT 20"
    )
    .bind(&q)
    .fetch_all(&state.db)
    .await
    .map_err(|e| format!("{e}"))?;

    Ok(SearchAllResults { tracks, albums, artists, genres, composers })
}

#[tauri::command]
pub async fn get_playlists_for_track(
    state: State<'_, AppState>,
    track_id: String,
) -> Result<Vec<String>, String> {
    playlist::get_playlists_for_track(&state.db, &track_id).await
}

use std::sync::Mutex;

static POWER_HANDLE: Mutex<Option<crate::power::PowerHandle>> = Mutex::new(None);

#[tauri::command]
pub async fn prevent_sleep() -> Result<(), String> {
    if let Ok(mut handle) = POWER_HANDLE.lock() {
        if handle.is_none() {
            *handle = crate::power::prevent_sleep();
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn allow_sleep() -> Result<(), String> {
    if let Ok(mut handle) = POWER_HANDLE.lock() {
        let h = handle.take();
        drop(handle);
        crate::power::allow_sleep(h);
    }
    Ok(())
}

#[tauri::command]
pub async fn get_artist_artwork(
    state: State<'_, AppState>,
    artist: String,
    size: Option<String>,
) -> Result<Option<Vec<u8>>, String> {
    let key = db::get_artist_artwork_key(&state.db, &artist).await?;
    if let Some(k) = key {
        let variant = size.as_deref();
        if state.artist_artwork_cache.exists(&k, variant) {
            return state.artist_artwork_cache.read(&k, variant).map(Some);
        }
        if variant.is_some() {
            if let Ok(data) = state.artist_artwork_cache.read(&k, None) {
                return Ok(Some(data));
            }
        }
        return Ok(None);
    }
    state.artist_artwork_worker.enqueue(artist).await;
    Ok(None)
}

#[tauri::command]
pub async fn remote_start(app_handle: tauri::AppHandle, state: State<'_, AppState>, port: u16, password: String) -> Result<(), String> {
    let queue = state.playlist_queue.clone();
    state.remote.start(
        app_handle,
        state.player.clone(),
        state.db.clone(),
        queue,
        state.volume.clone(),
        port,
        password,
    );
    Ok(())
}

#[tauri::command]
pub async fn remote_stop(state: State<'_, AppState>) -> Result<(), String> {
    state.remote.stop();
    Ok(())
}

#[tauri::command]
pub async fn remote_status(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.remote.is_running())
}

#[tauri::command]
pub async fn discord_connect(state: State<'_, AppState>, client_id: String) -> Result<(), String> {
    state.discord_rpc.connect(&client_id);
    Ok(())
}

#[tauri::command]
pub async fn discord_disconnect(state: State<'_, AppState>) -> Result<(), String> {
    state.discord_rpc.disconnect();
    Ok(())
}

#[tauri::command]
pub async fn update_discord_presence(
    state: State<'_, AppState>,
    state_str: String,
    details: String,
    start_time: Option<i64>,
    art_url: Option<String>,
) -> Result<(), String> {
    state.discord_rpc.update(&state_str, &details, start_time, art_url.as_deref());
    Ok(())
}

#[tauri::command]
pub async fn now_playing(
    _state: State<'_, AppState>,
    artist: String,
    track: String,
    album: String,
    duration: u32,
) -> Result<(), String> {
    let config = crate::lastfm::ScrobbleConfig {
        username: String::new(),
        session_key: String::new(),
        enabled: true,
    };
    let client = reqwest::Client::new();
    crate::lastfm::now_playing(&config, &artist, &track, &album, duration, &client).await
}

#[tauri::command]
pub async fn scrobble(
    _state: State<'_, AppState>,
    artist: String,
    track: String,
    album: String,
) -> Result<(), String> {
    let config = crate::lastfm::ScrobbleConfig {
        username: String::new(),
        session_key: String::new(),
        enabled: true,
    };
    let client = reqwest::Client::new();
    let timestamp = chrono::Utc::now().timestamp();
    crate::lastfm::scrobble(&config, &artist, &track, &album, timestamp, &client).await
}

#[tauri::command]
pub async fn get_radio_stations(state: State<'_, AppState>) -> Result<Vec<crate::radio::RadioStation>, String> {
    db::get_radio_stations(&state.db).await
}

#[tauri::command]
pub async fn add_radio_station(state: State<'_, AppState>, station: crate::radio::RadioStation) -> Result<(), String> {
    db::add_radio_station(&state.db, &station).await
}

#[tauri::command]
pub async fn delete_radio_station(state: State<'_, AppState>, id: String) -> Result<(), String> {
    db::delete_radio_station(&state.db, &id).await
}

#[tauri::command]
pub async fn update_radio_station(state: State<'_, AppState>, station: crate::radio::RadioStation) -> Result<(), String> {
    db::update_radio_station(&state.db, &station).await
}

#[tauri::command]
pub async fn play_radio(state: State<'_, AppState>, app_handle: tauri::AppHandle, url: String) -> Result<(), String> {
    if state.player.is_radio_active() {
        state.player.stop_radio();
    }
    state.player.play_radio(url, app_handle);
    Ok(())
}

#[tauri::command]
pub async fn ping_audio(state: State<'_, AppState>) -> Result<(), String> {
    state.player.ping()
}

#[tauri::command]
pub async fn stop_radio(state: State<'_, AppState>) -> Result<(), String> {
    state.player.stop_radio();
    Ok(())
}

#[tauri::command]
pub async fn radio_status(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.player.is_radio_active())
}

// --- EQ Profile Commands ---

#[derive(Serialize)]
pub struct EQProfileResult {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub is_active: bool,
    pub bands: Vec<f32>,
}

impl From<crate::library::eq::EQProfile> for EQProfileResult {
    fn from(p: crate::library::eq::EQProfile) -> Self {
        Self {
            id: p.id,
            name: p.name,
            is_default: p.is_default,
            is_active: p.is_active,
            bands: p.bands.to_vec(),
        }
    }
}

#[tauri::command]
pub async fn get_eq_profiles(state: State<'_, AppState>) -> Result<Vec<EQProfileResult>, String> {
    let profiles = crate::library::eq::get_all(&state.db).await?;
    Ok(profiles.into_iter().map(|p| p.into()).collect())
}

#[tauri::command]
pub async fn get_active_eq_profile(state: State<'_, AppState>) -> Result<Option<EQProfileResult>, String> {
    let profile = crate::library::eq::get_active(&state.db).await?;
    Ok(profile.map(|p| p.into()))
}

#[tauri::command]
pub async fn create_eq_profile(state: State<'_, AppState>, name: String) -> Result<EQProfileResult, String> {
    let profile = crate::library::eq::create(&state.db, &name).await?;
    Ok(profile.into())
}

#[tauri::command]
pub async fn rename_eq_profile(state: State<'_, AppState>, id: String, name: String) -> Result<(), String> {
    crate::library::eq::rename(&state.db, &id, &name).await
}

#[tauri::command]
pub async fn delete_eq_profile(state: State<'_, AppState>, id: String) -> Result<(), String> {
    crate::library::eq::delete(&state.db, &id).await
}

#[tauri::command]
pub async fn apply_eq_profile(state: State<'_, AppState>, id: String) -> Result<(), String> {
    crate::library::eq::set_active(&state.db, &id).await?;
    let profile = crate::library::eq::get_by_id(&state.db, &id)
        .await?
        .ok_or("Profile not found")?;
    for (i, gain) in profile.bands.iter().enumerate() {
        state.player.set_eq_band(i as u8, *gain);
    }
    state.player.set_eq_enabled(true);
    Ok(())
}

#[tauri::command]
pub async fn update_eq_band(state: State<'_, AppState>, profile_id: String, index: u8, gain_db: f32) -> Result<(), String> {
    crate::library::eq::update_band(&state.db, &profile_id, index, gain_db).await?;
    let profile = crate::library::eq::get_by_id(&state.db, &profile_id)
        .await?
        .ok_or("Profile not found")?;
    if profile.is_active {
        state.player.set_eq_band(index, gain_db.clamp(-12.0, 12.0));
    }
    Ok(())
}


