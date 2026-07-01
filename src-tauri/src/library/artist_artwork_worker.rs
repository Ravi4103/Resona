use crate::library::artist_artwork::ArtistArtworkCache;
use crate::library::db;
use sqlx::SqlitePool;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::Emitter;

pub struct ArtistArtworkWorker {
    queue: Arc<Mutex<Vec<String>>>,
    in_flight: Arc<Mutex<HashSet<String>>>,
    enabled: Arc<Mutex<bool>>,
}

impl ArtistArtworkWorker {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(Vec::new())),
            in_flight: Arc::new(Mutex::new(HashSet::new())),
            enabled: Arc::new(Mutex::new(true)),
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        *self.enabled.blocking_lock() = enabled;
    }

    pub async fn enqueue(&self, artist: String) {
        let mut in_flight = self.in_flight.lock().await;
        if in_flight.contains(&artist) {
            return;
        }
        in_flight.insert(artist.clone());
        drop(in_flight);
        let mut queue = self.queue.lock().await;
        queue.push(artist);
    }

    pub fn start(
        self: Arc<Self>,
        db: SqlitePool,
        cache: Arc<ArtistArtworkCache>,
        app_handle: tauri::AppHandle,
    ) {
        tauri::async_runtime::spawn(async move {
            loop {
                let artist = {
                    let mut queue = self.queue.lock().await;
                    if queue.is_empty() {
                        None
                    } else {
                        Some(queue.remove(0))
                    }
                };
                if let Some(artist_name) = artist {
                    let enabled = *self.enabled.lock().await;
                    if enabled {
                        Self::process(&db, &cache, &app_handle, &artist_name).await;
                    }
                    let mut in_flight = self.in_flight.lock().await;
                    in_flight.remove(&artist_name);
                }
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        });
    }

    async fn process(
        db: &SqlitePool,
        cache: &ArtistArtworkCache,
        app_handle: &tauri::AppHandle,
        artist_name: &str,
    ) {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(8))
            .build()
            .ok();
        let client = match client {
            Some(c) => c,
            None => return,
        };

        let url = format!(
            "https://api.deezer.com/search/artist?q={}",
            urlencoding(artist_name)
        );

        let response = match client.get(&url).send().await {
            Ok(r) => r,
            Err(_) => return,
        };

        let data: serde_json::Value = match response.json().await {
            Ok(d) => d,
            Err(_) => return,
        };

        let picture_url = match data["data"][0]["picture_medium"].as_str() {
            Some(u) => u.to_string(),
            None => return,
        };

        let img_response = match client.get(&picture_url).send().await {
            Ok(r) => r,
            Err(_) => return,
        };

        let img_bytes = match img_response.bytes().await {
            Ok(b) => b.to_vec(),
            Err(_) => return,
        };

        let key = match cache.save(&img_bytes) {
            Ok(k) => k,
            Err(_) => return,
        };

        let _ = db::set_artist_artwork_key(db, artist_name, &key).await;

        let _ = app_handle.emit("artist-artwork-updated", serde_json::json!({
            "artist": artist_name,
            "key": key,
        }));
    }
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
