use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

pub const EQ_FREQUENCIES: [f32; 10] = [
    32.0, 64.0, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0,
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EQProfile {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub is_active: bool,
    pub bands: [f32; 10],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EQBandUpdate {
    pub index: u8,
    pub gain_db: f32,
}

struct DefaultPreset {
    name: &'static str,
    gains: [f32; 10],
}

const DEFAULT_PRESETS: &[DefaultPreset] = &[
    DefaultPreset { name: "Flat", gains: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0] },
    DefaultPreset { name: "Rock", gains: [4.0, 3.0, 2.0, 1.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0] },
    DefaultPreset { name: "Pop", gains: [2.0, 3.0, 4.0, 5.0, 3.0, 0.0, 1.0, 2.0, 3.0, 2.0] },
    DefaultPreset { name: "Jazz", gains: [3.0, 2.0, 1.0, 2.0, 3.0, 2.0, 1.0, 2.0, 3.0, 4.0] },
    DefaultPreset { name: "Classical", gains: [4.0, 3.0, 2.0, 1.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0] },
    DefaultPreset { name: "Dance/EDM", gains: [4.0, 3.0, 2.0, 0.0, -1.0, 0.0, 2.0, 3.0, 4.0, 5.0] },
    DefaultPreset { name: "Acoustic", gains: [3.0, 3.0, 2.0, 1.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0] },
    DefaultPreset { name: "Bass Boost", gains: [5.0, 4.0, 3.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0] },
    DefaultPreset { name: "Vocal Boost", gains: [0.0, 0.0, 0.0, 2.0, 4.0, 4.0, 2.0, 1.0, 0.0, 0.0] },
    DefaultPreset { name: "Headphones", gains: [3.0, 2.0, 1.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0] },
    DefaultPreset { name: "Loudness", gains: [3.0, 2.0, 1.0, 2.0, 3.0, 3.0, 2.0, 1.0, 2.0, 3.0] },
    DefaultPreset { name: "Podcast", gains: [0.0, 0.0, 0.0, 2.0, 4.0, 4.0, 2.0, 0.0, -1.0, -1.0] },
];

pub async fn seed_defaults(pool: &SqlitePool) -> Result<(), String> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM eq_profiles")
        .fetch_one(pool)
        .await
        .map_err(|e| format!("{e}"))?;
    if count.0 > 0 {
        return Ok(());
    }
    for (i, preset) in DEFAULT_PRESETS.iter().enumerate() {
        let id = uuid::Uuid::new_v4().to_string();
        let is_active = if i == 0 { 1 } else { 0 };
        sqlx::query("INSERT INTO eq_profiles (id, name, is_default, is_active) VALUES (?, ?, 1, ?)")
            .bind(&id)
            .bind(preset.name)
            .bind(is_active)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to seed preset {}: {e}", preset.name))?;
        for (band_idx, gain) in preset.gains.iter().enumerate() {
            sqlx::query("INSERT INTO eq_bands (profile_id, band_index, gain) VALUES (?, ?, ?)")
                .bind(&id)
                .bind(band_idx as i32)
                .bind(gain)
                .execute(pool)
                .await
                .map_err(|e| format!("Failed to seed band: {e}"))?;
        }
    }
    Ok(())
}

#[derive(sqlx::FromRow)]
struct ProfileRow {
    id: String,
    name: String,
    is_default: bool,
    is_active: bool,
}

async fn load_bands(pool: &SqlitePool, profile_id: &str) -> Result<[f32; 10], String> {
    #[derive(sqlx::FromRow)]
    struct BandRow {
        band_index: i32,
        gain: f32,
    }
    let rows: Vec<BandRow> = sqlx::query_as(
        "SELECT band_index, gain FROM eq_bands WHERE profile_id = ? ORDER BY band_index"
    )
    .bind(profile_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("{e}"))?;
    let mut bands = [0.0f32; 10];
    for row in rows {
        if row.band_index >= 0 && (row.band_index as usize) < 10 {
            bands[row.band_index as usize] = row.gain;
        }
    }
    Ok(bands)
}

fn row_to_profile(row: ProfileRow, bands: [f32; 10]) -> EQProfile {
    EQProfile {
        id: row.id,
        name: row.name,
        is_default: row.is_default,
        is_active: row.is_active,
        bands,
    }
}

pub async fn get_all(pool: &SqlitePool) -> Result<Vec<EQProfile>, String> {
    let rows: Vec<ProfileRow> = sqlx::query_as(
        "SELECT id, name, is_default, is_active FROM eq_profiles ORDER BY is_default DESC, created_at ASC"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("{e}"))?;
    let mut profiles = Vec::new();
    for row in rows {
        let bands = load_bands(pool, &row.id).await?;
        profiles.push(row_to_profile(row, bands));
    }
    Ok(profiles)
}

pub async fn get_active(pool: &SqlitePool) -> Result<Option<EQProfile>, String> {
    let row: Option<ProfileRow> = sqlx::query_as(
        "SELECT id, name, is_default, is_active FROM eq_profiles WHERE is_active = 1 LIMIT 1"
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("{e}"))?;
    match row {
        Some(r) => {
            let bands = load_bands(pool, &r.id).await?;
            Ok(Some(row_to_profile(r, bands)))
        }
        None => Ok(None),
    }
}

pub async fn get_by_id(pool: &SqlitePool, id: &str) -> Result<Option<EQProfile>, String> {
    let row: Option<ProfileRow> = sqlx::query_as(
        "SELECT id, name, is_default, is_active FROM eq_profiles WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("{e}"))?;
    match row {
        Some(r) => {
            let bands = load_bands(pool, &r.id).await?;
            Ok(Some(row_to_profile(r, bands)))
        }
        None => Ok(None),
    }
}

pub async fn create(pool: &SqlitePool, name: &str) -> Result<EQProfile, String> {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO eq_profiles (id, name, is_default, is_active) VALUES (?, ?, 0, 0)")
        .bind(&id)
        .bind(name)
        .execute(pool)
        .await
        .map_err(|e| format!("{e}"))?;
    for i in 0..10 {
        sqlx::query("INSERT INTO eq_bands (profile_id, band_index, gain) VALUES (?, ?, 0.0)")
            .bind(&id)
            .bind(i as i32)
            .execute(pool)
            .await
            .map_err(|e| format!("{e}"))?;
    }
    Ok(EQProfile {
        id,
        name: name.to_string(),
        is_default: false,
        is_active: false,
        bands: [0.0; 10],
    })
}

pub async fn rename(pool: &SqlitePool, id: &str, name: &str) -> Result<(), String> {
    let affected = sqlx::query("UPDATE eq_profiles SET name = ? WHERE id = ? AND is_default = 0")
        .bind(name)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| format!("{e}"))?
        .rows_affected();
    if affected == 0 {
        return Err("Profile not found or is a default preset".into());
    }
    Ok(())
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), String> {
    let profile: Option<ProfileRow> = sqlx::query_as(
        "SELECT id, name, is_default, is_active FROM eq_profiles WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("{e}"))?;
    match profile {
        Some(p) if p.is_default => Err("Cannot delete a default preset".into()),
        Some(_) => {
            sqlx::query("DELETE FROM eq_profiles WHERE id = ?")
                .bind(id)
                .execute(pool)
                .await
                .map_err(|e| format!("{e}"))?;
            Ok(())
        }
        None => Err("Profile not found".into()),
    }
}

pub async fn set_active(pool: &SqlitePool, id: &str) -> Result<(), String> {
    sqlx::query("UPDATE eq_profiles SET is_active = 0")
        .execute(pool)
        .await
        .map_err(|e| format!("{e}"))?;
    sqlx::query("UPDATE eq_profiles SET is_active = 1 WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| format!("{e}"))?;
    Ok(())
}

pub async fn update_band(pool: &SqlitePool, profile_id: &str, index: u8, gain_db: f32) -> Result<(), String> {
    let gain = gain_db.clamp(-12.0, 12.0);
    sqlx::query("UPDATE eq_bands SET gain = ? WHERE profile_id = ? AND band_index = ?")
        .bind(gain)
        .bind(profile_id)
        .bind(index as i32)
        .execute(pool)
        .await
        .map_err(|e| format!("{e}"))?;
    Ok(())
}
