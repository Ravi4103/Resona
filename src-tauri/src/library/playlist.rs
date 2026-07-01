use crate::library::track::Track;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: String,
}

pub async fn create_playlist(pool: &SqlitePool, name: &str) -> Result<Playlist, String> {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO playlists (id, name) VALUES (?, ?)")
        .bind(&id)
        .bind(name)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to create playlist: {e}"))?;
    Ok(Playlist {
        id,
        name: name.to_string(),
        description: String::new(),
        created_at: String::new(),
    })
}

pub async fn rename_playlist(pool: &SqlitePool, id: &str, name: &str) -> Result<(), String> {
    sqlx::query("UPDATE playlists SET name = ? WHERE id = ?")
        .bind(name)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to rename playlist: {e}"))?;
    Ok(())
}

pub async fn delete_playlist(pool: &SqlitePool, id: &str) -> Result<(), String> {
    sqlx::query("DELETE FROM playlists WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to delete playlist: {e}"))?;
    Ok(())
}

pub async fn get_playlists(pool: &SqlitePool) -> Result<Vec<Playlist>, String> {
    let rows = sqlx::query_as::<_, PlaylistRow>("SELECT id, name, description, created_at FROM playlists ORDER BY name")
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to get playlists: {e}"))?;
    Ok(rows.into_iter().map(|r| Playlist {
        id: r.id,
        name: r.name,
        description: r.description,
        created_at: r.created_at,
    }).collect())
}

pub async fn add_track_to_playlist(pool: &SqlitePool, playlist_id: &str, track_id: &str) -> Result<(), String> {
    let max_pos: Option<i32> = sqlx::query_scalar(
        "SELECT MAX(position) FROM playlist_tracks WHERE playlist_id = ?"
    )
    .bind(playlist_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Failed to get max position: {e}"))?;
    let pos = max_pos.unwrap_or(0) + 1;
    sqlx::query("INSERT OR IGNORE INTO playlist_tracks (playlist_id, track_id, position) VALUES (?, ?, ?)")
        .bind(playlist_id)
        .bind(track_id)
        .bind(pos)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to add track to playlist: {e}"))?;
    Ok(())
}

pub async fn remove_track_from_playlist(pool: &SqlitePool, playlist_id: &str, track_id: &str) -> Result<(), String> {
    sqlx::query("DELETE FROM playlist_tracks WHERE playlist_id = ? AND track_id = ?")
        .bind(playlist_id)
        .bind(track_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to remove track: {e}"))?;
    Ok(())
}

pub async fn get_playlists_for_track(pool: &SqlitePool, track_id: &str) -> Result<Vec<String>, String> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT playlist_id FROM playlist_tracks WHERE track_id = ?"
    )
    .bind(track_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to get playlists for track: {e}"))?;
    Ok(rows.into_iter().map(|r| r.0).collect())
}

pub async fn get_playlist_tracks(pool: &SqlitePool, playlist_id: &str) -> Result<Vec<Track>, String> {
    let rows = sqlx::query_as::<_, crate::library::db::TrackRow>(
         "SELECT t.id, t.file_path, t.title, t.artist, t.album, t.album_artist, t.composer,
          t.track_number, t.disc_number, t.year, t.genre, t.raw_genre_names, t.duration,
          t.sample_rate, t.bit_depth, t.file_size, t.file_format, t.has_artwork,
          t.replaygain_track_gain, t.replaygain_album_gain, t.cue_parent_id, t.cue_offset, t.artwork_key
          FROM tracks t
          JOIN playlist_tracks pt ON pt.track_id = t.id
          WHERE pt.playlist_id = ?
          ORDER BY pt.position"
    )
    .bind(playlist_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to get playlist tracks: {e}"))?;
    Ok(rows.into_iter().map(|r| r.into()).collect())
}

pub async fn import_playlist_m3u(pool: &SqlitePool, file_path: &str, name: &str) -> Result<Playlist, String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read M3U file: {e}"))?;

    let playlist = create_playlist(pool, if name.is_empty() { "Imported Playlist" } else { name }).await?;

    // Check existing tracks by file_path for quick lookup
    let existing: std::collections::HashMap<String, String> = sqlx::query_as::<_, (String, String)>(
        "SELECT id, file_path FROM tracks"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("{e}"))?
    .into_iter()
    .map(|(id, path)| (path, id))
    .collect();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Try to find track by path (normalize separators)
        let normalized = line.replace('/', "\\");
        if let Some(track_id) = existing.get(line).or_else(|| existing.get(&normalized)) {
            let _ = add_track_to_playlist(pool, &playlist.id, track_id).await;
        }
    }

    Ok(playlist)
}

pub async fn export_playlist_m3u(pool: &SqlitePool, id: &str, output_path: &str) -> Result<(), String> {
    let tracks = get_playlist_tracks(pool, id).await?;
    let playlist_name = sqlx::query_scalar::<_, String>("SELECT name FROM playlists WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to get playlist name: {e}"))?
        .unwrap_or_default();

    let mut content = format!("#EXTM3U\n#PLAYLIST:{playlist_name}\n");
    for t in &tracks {
        content.push_str(&format!("#EXTINF:{},{}\n", t.duration.round() as u64, t.title));
        content.push_str(&format!("{}\n", t.file_path));
    }

    fs::write(output_path, content).map_err(|e| format!("Failed to write M3U: {e}"))?;
    Ok(())
}

#[derive(sqlx::FromRow)]
struct PlaylistRow {
    id: String,
    name: String,
    description: String,
    created_at: String,
}
