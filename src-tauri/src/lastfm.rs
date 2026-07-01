use serde::{Deserialize, Serialize};

const API_KEY: &str = "4a1b7a2c3d4e5f678901234567890abc";
const SHARED_SECRET: &str = "0123456789abcdef0123456789abcdef";
const BASE_URL: &str = "https://ws.audioscrobbler.com/2.0/";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrobbleConfig {
    pub username: String,
    pub session_key: String,
    pub enabled: bool,
}

pub async fn now_playing(
    config: &ScrobbleConfig,
    artist: &str,
    track: &str,
    album: &str,
    duration: u32,
    client: &reqwest::Client,
) -> Result<(), String> {
    if !config.enabled || config.username.is_empty() {
        return Ok(());
    }
    let params = [
        ("method", "track.updateNowPlaying"),
        ("artist", artist),
        ("track", track),
        ("album", album),
        ("duration", &duration.to_string()),
        ("api_key", API_KEY),
        ("sk", &config.session_key),
    ];
    send_request(client, &params).await
}

pub async fn scrobble(
    config: &ScrobbleConfig,
    artist: &str,
    track: &str,
    album: &str,
    timestamp: i64,
    client: &reqwest::Client,
) -> Result<(), String> {
    if !config.enabled || config.username.is_empty() {
        return Ok(());
    }
    let params = [
        ("method", "track.scrobble"),
        ("artist", artist),
        ("track", track),
        ("album", album),
        ("timestamp", &timestamp.to_string()),
        ("api_key", API_KEY),
        ("sk", &config.session_key),
    ];
    send_request(client, &params).await
}

fn sign(params: &[(&str, &str)]) -> String {
    let mut sorted: Vec<(&str, &str)> = params.to_vec();
    sorted.sort_by_key(|(k, _)| *k);
    let concat: String = sorted
        .iter()
        .map(|(k, v)| format!("{}{}", k, v))
        .collect::<Vec<_>>()
        .join("");
    let to_sign = format!("{}{}", concat, SHARED_SECRET);
    use md5::{Md5, Digest};
    let digest = Md5::digest(to_sign.as_bytes());
    format!("{:x}", digest)
}

async fn send_request(
    client: &reqwest::Client,
    params: &[(&str, &str)],
) -> Result<(), String> {
    let signature = sign(params);

    let mut form = Vec::new();
    for (k, v) in params {
        form.push((*k, *v));
    }
    form.push(("api_sig", signature.as_str()));
    form.push(("format", "json"));

    let resp = client
        .post(BASE_URL)
        .form(&form)
        .send()
        .await
        .map_err(|e| format!("Last.fm request failed: {e}"))?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        log::warn!("Last.fm API error: {body}");
    }

    Ok(())
}
