use sha2::{Digest, Sha256};
use std::path::PathBuf;

pub struct ArtistArtworkCache {
    base_path: PathBuf,
}

impl ArtistArtworkCache {
    pub fn new(base_path: PathBuf) -> Self {
        std::fs::create_dir_all(&base_path).ok();
        Self { base_path }
    }

    fn key_for_data(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    pub fn save(&self, data: &[u8]) -> Result<String, String> {
        let key = Self::key_for_data(data);
        let orig_path = self.path_for_key(&key, None);
        if orig_path.exists() {
            return Ok(key);
        }
        std::fs::write(&orig_path, data).map_err(|e| format!("Failed to write artwork: {e}"))?;
        if let Ok(img) = image::load_from_memory(data) {
            let sm = img.thumbnail(64, 64);
            let sm_path = self.path_for_key(&key, Some("sm"));
            let _ = sm.save(&sm_path);
            let md = img.thumbnail(500, 500);
            let md_path = self.path_for_key(&key, Some("md"));
            let _ = md.save(&md_path);
        }
        Ok(key)
    }

    pub fn path_for_key(&self, key: &str, variant: Option<&str>) -> PathBuf {
        let filename = match variant {
            Some(v) => format!("{key}_{v}.jpg"),
            None => format!("{key}.jpg"),
        };
        self.base_path.join(filename)
    }

    pub fn read(&self, key: &str, variant: Option<&str>) -> Result<Vec<u8>, String> {
        let path = self.path_for_key(key, variant);
        std::fs::read(&path).map_err(|e| format!("Failed to read artwork: {e}"))
    }

    pub fn exists(&self, key: &str, variant: Option<&str>) -> bool {
        self.path_for_key(key, variant).exists()
    }
}
