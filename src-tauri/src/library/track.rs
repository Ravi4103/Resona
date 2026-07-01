use lofty::file::AudioFile;
use lofty::file::TaggedFileExt;
use lofty::read_from_path;
use lofty::tag::{Accessor, ItemKey, Tag};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub fn extract_lyrics_from_file(path: &Path) -> Option<(String, String)> {
    let tagged_file = read_from_path(path).ok()?;
    let tag = tagged_file.tags().first()?;
    let lyrics = tag.get_string(&ItemKey::Lyrics)?;
    let text = lyrics.trim().to_string();
    if text.is_empty() { return None; }
    let source = if text.contains('[') && text.contains(']') { "meta-synced" } else { "meta-plain" };
    Some((text, source.to_string()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub file_path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: String,
    pub composer: String,
    pub track_number: u32,
    pub disc_number: u32,
    pub year: i32,
    pub genre: String,
    pub raw_genre_names: String,
    pub duration: f64,
    pub sample_rate: u32,
    pub bit_depth: u32,
    pub file_size: u64,
    pub file_format: String,
    pub has_artwork: bool,
    pub replaygain_track_gain: f64,
    pub replaygain_album_gain: f64,
    pub cue_parent_id: Option<String>,
    pub cue_offset: f64,
    pub artwork_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genre {
    pub id: String,
    pub name: String,
    pub normalization_key: String,
    pub track_count: i64,
}

pub fn extract_artwork_bytes(path: &Path) -> Option<Vec<u8>> {
    let tagged_file = read_from_path(path).ok()?;
    let picture = tagged_file.tags().iter().filter_map(|tag| tag.pictures().first()).next()?;
    Some(picture.data().to_vec())
}

/// Parse a ReplayGain string like "-5.34 dB" into a float value
fn parse_replaygain(value: &str) -> f64 {
    let s = value.trim().to_lowercase();
    let s = s.trim_end_matches("db").trim();
    s.parse::<f64>().unwrap_or(0.0)
}

impl Track {
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let file_size = std::fs::metadata(path)
            .map(|m| m.len())
            .unwrap_or(0);
        let file_format = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let tagged_file = read_from_path(path).map_err(|e| format!("Failed to read metadata: {e}"))?;
        let properties = tagged_file.properties();
        let tag = tagged_file.tags().first().cloned();
        let title = tag
            .as_ref()
            .and_then(|t| t.title())
            .map(|s| s.to_string())
            .unwrap_or_default();
        let artist = tag
            .as_ref()
            .and_then(|t| t.artist())
            .map(|s| s.to_string())
            .unwrap_or_default();
        let album = tag
            .as_ref()
            .and_then(|t| t.album())
            .map(|s| s.to_string())
            .unwrap_or_default();
        let album_artist = tag
            .as_ref()
            .and_then(|t| t.artist())
            .map(|s| s.to_string())
            .unwrap_or_default();
        let composer = tag
            .as_ref()
            .map(|t| extract_composer(t))
            .unwrap_or_default();
        let genre = tag
            .as_ref()
            .and_then(|t| t.genre())
            .map(|s| s.to_string())
            .unwrap_or_default();
        let track_number = tag
            .as_ref()
            .and_then(|t| t.track())
            .unwrap_or(0);
        let disc_number = tag
            .as_ref()
            .and_then(|t| t.disk())
            .unwrap_or(0);
        let year = tag
            .as_ref()
            .and_then(|t| t.year())
            .unwrap_or(0) as i32;
        let duration = properties.duration().as_secs_f64();
        let sample_rate = properties.sample_rate().unwrap_or(0) as u32;
        let bit_depth = properties.bit_depth().unwrap_or(0) as u32;
        let has_artwork = tag
            .as_ref()
            .map(|t| !t.pictures().is_empty())
            .unwrap_or(false);
        let replaygain_track_gain = tag
            .as_ref()
            .and_then(|t| t.get_string(&ItemKey::ReplayGainTrackGain))
            .map(parse_replaygain)
            .unwrap_or(0.0);
        let replaygain_album_gain = tag
            .as_ref()
            .and_then(|t| t.get_string(&ItemKey::ReplayGainAlbumGain))
            .map(parse_replaygain)
            .unwrap_or(0.0);
        let raw_genre_names = extract_raw_genres(&tagged_file);
        Ok(Self {
            id: uuid::Uuid::new_v4().to_string(),
            file_path: path.to_string_lossy().to_string(),
            title,
            artist,
            album,
            album_artist,
            composer,
            track_number,
            disc_number,
            year,
            genre,
            duration,
            sample_rate,
            bit_depth,
            file_size,
            file_format,
            has_artwork,
            replaygain_track_gain,
            replaygain_album_gain,
            cue_parent_id: None,
            cue_offset: 0.0,
            artwork_key: None,
            raw_genre_names,
        })
    }
}

/// Extract raw genre names from all tags.
/// Collects all genre items, joins with "; " to preserve original tag formatting.
fn extract_raw_genres(tagged_file: &(impl AudioFile + TaggedFileExt)) -> String {
    let mut values: Vec<String> = Vec::new();
    for tag in tagged_file.tags() {
        for item in tag.items() {
            if item.key() == &ItemKey::Genre {
                if let Some(text) = item.value().text() {
                    let trimmed = text.trim().to_string();
                    if !trimmed.is_empty() && !values.contains(&trimmed) {
                        values.push(trimmed);
                    }
                }
            }
        }
    }
    values.join("; ")
}

/// Normalize a genre name for dedup (lowercase, trim).
pub fn normalize_genre_name(name: &str) -> String {
    name.trim().to_lowercase()
}

/// Split a raw genre string into individual genre names using common delimiters.
/// Matches Airmedy's SplitArtists-like behavior for genres.
pub fn split_genre_string(raw: &str) -> Vec<String> {
    if raw.is_empty() {
        return Vec::new();
    }
    let mut results: Vec<String> = Vec::new();
    // Split on common delimiters: ; , / |
    for part in raw.split(|c: char| c == ';' || c == ',' || c == '/' || c == '|') {
        let trimmed = part.trim().to_string();
        if !trimmed.is_empty() && !results.contains(&trimmed) {
            results.push(trimmed);
        }
    }
    results
}

/// Extract composer string from tag, trying multiple known tag keys (like Airmedy).
/// Falls back to scanning all items for composer-related keys.
fn extract_composer(tag: &Tag) -> String {
    // Try high-level key first
    if let Some(val) = tag.get_string(&ItemKey::Composer) {
        let trimmed = val.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    // Fallback: scan all items for composer-related keys using Debug representation.
    // This catches cases where the key was stored as Unknown("TCOM") or similar.
    let mut composers: Vec<String> = Vec::new();
    for item in tag.items() {
        let key_repr = format!("{:?}", item.key());
        let upper = key_repr.to_uppercase();
        let is_composer = upper.contains("COMPOSER")
            || upper.contains("©WRT")
            || upper == "\"TCOM\""
            || upper.contains("\"WRITER\"")
            || upper == "\"WRITER\"";
        if is_composer {
            if let Some(text) = item.value().text() {
                let trimmed = text.trim();
                if !trimmed.is_empty() && !composers.contains(&trimmed.to_string()) {
                    composers.push(trimmed.to_string());
                }
            }
        }
    }

    composers.join("; ")
}
