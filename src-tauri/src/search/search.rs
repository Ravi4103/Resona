use crate::library::track::Track;
use std::collections::HashMap;
use std::path::Path;

pub struct SearchIndex {
    tracks: HashMap<String, Track>,
}

impl SearchIndex {
    pub fn new(_path: &Path) -> Result<Self, String> {
        Ok(Self {
            tracks: HashMap::new(),
        })
    }

    pub fn index_track(&mut self, track: &Track) {
        self.tracks.insert(track.id.clone(), track.clone());
    }

    pub fn remove_track(&mut self, track_id: &str) {
        self.tracks.remove(track_id);
    }

    pub fn rebuild(&mut self, tracks: &[Track]) {
        self.tracks.clear();
        for t in tracks {
            self.tracks.insert(t.id.clone(), t.clone());
        }
    }

    pub fn search(&self, query: &str, _limit: usize) -> Vec<String> {
        let q = query.to_lowercase();
        let mut results: Vec<(f32, String)> = self
            .tracks
            .values()
            .filter_map(|t| {
                let mut score = 0.0f32;
                if t.title.to_lowercase().contains(&q) {
                    score += 10.0;
                }
                if t.artist.to_lowercase().contains(&q) {
                    score += 8.0;
                }
                if t.album.to_lowercase().contains(&q) {
                    score += 6.0;
                }
                if t.raw_genre_names.to_lowercase().contains(&q) {
                    score += 4.0;
                }
                if t.file_path.to_lowercase().contains(&q) {
                    score += 2.0;
                }
                if score > 0.0 {
                    Some((score, t.id.clone()))
                } else {
                    None
                }
            })
            .collect();
        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        results.into_iter().map(|(_, id)| id).collect()
    }
}
