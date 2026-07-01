use std::path::Path;

/// A parsed CUE track entry
#[derive(Debug, Clone)]
pub struct CueTrack {
    pub title: String,
    pub artist: String,
    pub track_number: u32,
    pub start_secs: f64,
}

/// Parse a timestamp like "03:45:30" (MM:SS:FF, 75 fps) into seconds
fn parse_timestamp(ts: &str) -> Option<f64> {
    let parts: Vec<&str> = ts.split(':').collect();
    if parts.len() != 3 {
        return None;
    }
    let minutes: f64 = parts[0].parse().ok()?;
    let seconds: f64 = parts[1].parse().ok()?;
    let frames: f64 = parts[2].parse().ok()?;
    Some(minutes * 60.0 + seconds + frames / 75.0)
}

/// Parse a CUE sheet string into a list of tracks with their start offsets
pub fn parse_cue(content: &str) -> Vec<CueTrack> {
    let mut tracks: Vec<CueTrack> = Vec::new();
    let mut current_track: Option<CueTrack> = None;
    let mut performer = String::new();

    for line in content.lines() {
        let line = line.trim();

        if line.to_uppercase().starts_with("PERFORMER ") {
            let val = extract_quoted_value(line);
            if let Some(v) = val {
                performer = v;
            }
        }

        if line.to_uppercase().starts_with("TRACK ") {
            // Push previous track if exists
            if let Some(t) = current_track.take() {
                tracks.push(t);
            }
            let num_part = line.trim_start_matches("TRACK ").trim();
            let num: u32 = num_part.split_whitespace().next().and_then(|s| s.parse().ok()).unwrap_or(0);
            current_track = Some(CueTrack {
                title: String::new(),
                artist: performer.clone(),
                track_number: num,
                start_secs: 0.0,
            });
        }

        if line.to_uppercase().starts_with("TITLE ") {
            let val = extract_quoted_value(line);
            if let Some(v) = val {
                if let Some(ref mut t) = current_track {
                    t.title = v;
                } else {
                    // Album title, ignore
                }
            }
        }

        if line.to_uppercase().starts_with("PERFORMER ") {
            let val = extract_quoted_value(line);
            if let Some(v) = val {
                if let Some(ref mut t) = current_track.as_mut() {
                    t.artist = v;
                }
            }
        }

        if line.to_uppercase().starts_with("INDEX 01") || line.to_uppercase().starts_with("INDEX 1") {
            let ts_part = line.split_whitespace().last().unwrap_or("");
            if let Some(secs) = parse_timestamp(ts_part) {
                if let Some(ref mut t) = current_track {
                    t.start_secs = secs;
                }
            }
        }
    }

    // Push last track
    if let Some(t) = current_track.take() {
        tracks.push(t);
    }

    // Deduplicate by track number (keep last occurrence)
    let mut seen = std::collections::HashSet::new();
    tracks.reverse();
    tracks.retain(|t| seen.insert(t.track_number));
    tracks.reverse();

    tracks
}

/// Find companion .cue files for an audio file
pub fn find_cue_for_audio(audio_path: &Path) -> Option<String> {
    let cue_path = audio_path.with_extension("cue");
    if cue_path.exists() {
        return std::fs::read_to_string(&cue_path).ok();
    }

    // Also check for .cue in the same directory with same stem
    if let Some(parent) = audio_path.parent() {
        let stem = audio_path.file_stem()?;
        for entry in std::fs::read_dir(parent).ok()? {
            if let Ok(e) = entry {
                let p = e.path();
                if p.extension().and_then(|e| e.to_str()) == Some("cue") {
                    if let Ok(content) = std::fs::read_to_string(&p) {
                        // Check if this CUE references our file
                        for line in content.lines() {
                            if line.to_uppercase().contains("FILE") {
                                if line.to_lowercase().contains(&stem.to_string_lossy().to_lowercase()) {
                                    return Some(content);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

fn extract_quoted_value(line: &str) -> Option<String> {
    // Find first quote
    let start = line.find('"')?;
    let rest = &line[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}
