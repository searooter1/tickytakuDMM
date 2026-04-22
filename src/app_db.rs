use serde::{Deserialize, Serialize};
use std::path::Path;
use surrealdb::engine::local::SurrealKv;
use surrealdb::types::SurrealValue;
use surrealdb::Surreal;

/// Mod library metadata stored in embedded SurrealDB (not Deadlock `citadel/addons` paths).
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
#[surreal(crate = "surrealdb::types")]
pub struct ModMetadataRecord {
    /// Stable id for this metadata row (not the Surreal record id; that is `file_name`).
    pub metadata_id: uuid::Uuid,
    pub file_name: String,
    pub title: String,
    pub description: Option<String>,
    /// Stored as a string path so the value round-trips through SurrealDB.
    pub thumbnail_path: Option<String>,
}

/// Shape of legacy `mod_metadata.json` (paths as JSON strings).
#[derive(Debug, Deserialize)]
struct LegacyModMetadata {
    id: uuid::Uuid,
    file_name: String,
    title: String,
    description: Option<String>,
    thumbnail_path: Option<std::path::PathBuf>,
}

impl From<LegacyModMetadata> for ModMetadataRecord {
    fn from(legacy: LegacyModMetadata) -> Self {
        Self {
            metadata_id: legacy.id,
            file_name: legacy.file_name,
            title: legacy.title,
            description: legacy.description,
            thumbnail_path: legacy
                .thumbnail_path
                .map(|p| p.to_string_lossy().into_owned()),
        }
    }
}

const TABLE: &str = "mod_metadata";
const NS: &str = "tickytaku";
const DB: &str = "dmm";

pub type Db = Surreal<surrealdb::engine::local::Db>;

pub async fn open(base_data_dir: &Path) -> surrealdb::Result<Db> {
    let db_path = base_data_dir.join("surreal");
    let db = Surreal::new::<SurrealKv>(db_path).await?;
    db.use_ns(NS).use_db(DB).await?;
    db.query("DEFINE TABLE IF NOT EXISTS mod_metadata SCHEMALESS;")
        .await?
        .check()?;
    Ok(db)
}

/// Import legacy `mod_metadata.json` once when the Surreal table is still empty.
pub async fn migrate_from_json_if_needed(
    db: &Db,
    legacy_json_path: &Path,
) -> Result<(), String> {
    let existing: Vec<ModMetadataRecord> = db
        .select(TABLE)
        .await
        .map_err(|e| format!("Could not read metadata from database: {e}"))?;

    if !existing.is_empty() {
        return Ok(());
    }

    if !legacy_json_path.exists() {
        return Ok(());
    }

    let text = std::fs::read_to_string(legacy_json_path)
        .map_err(|e| format!("Could not read legacy metadata file: {e}"))?;

    let legacy: Vec<LegacyModMetadata> = serde_json::from_str(&text)
        .map_err(|e| format!("Could not parse legacy metadata file: {e}"))?;

    for entry in legacy.into_iter().map(ModMetadataRecord::from) {
        let key = entry.file_name.as_str();
        let _: Option<ModMetadataRecord> = db
            .upsert((TABLE, key))
            .content(entry)
            .await
            .map_err(|e| format!("Could not migrate metadata row: {e}"))?;
    }

    let backup = legacy_json_path.with_extension("json.migrated.bak");
    std::fs::rename(legacy_json_path, &backup).map_err(|e| {
        format!(
            "Migrated metadata to SurrealDB but could not rename legacy file to {}: {e}",
            backup.display()
        )
    })?;

    Ok(())
}

pub async fn load_all_metadata(db: &Db) -> Result<Vec<ModMetadataRecord>, String> {
    db.select(TABLE)
        .await
        .map_err(|e| format!("Could not read metadata from database: {e}"))
}

pub async fn upsert_metadata(db: &Db, record: ModMetadataRecord) -> Result<(), String> {
    let key = record.file_name.clone();
    let _: Option<ModMetadataRecord> = db
        .upsert((TABLE, key.as_str()))
        .content(record)
        .await
        .map_err(|e| format!("Could not save metadata: {e}"))?;
    Ok(())
}

pub async fn delete_metadata(db: &Db, file_name: &str) -> Result<(), String> {
    let _: Option<ModMetadataRecord> = db
        .delete((TABLE, file_name))
        .await
        .map_err(|e| format!("Could not delete metadata: {e}"))?;
    Ok(())
}

/// Surreal record id is `file_name`; re-key when the on-disk `.vpk` is renamed.
pub async fn rename_metadata_file_name(
    db: &Db,
    old_file_name: &str,
    new_file_name: &str,
) -> Result<(), String> {
    if old_file_name == new_file_name {
        return Ok(());
    }

    let existing: Option<ModMetadataRecord> = db
        .select((TABLE, old_file_name))
        .await
        .map_err(|e| format!("Could not read metadata for rename: {e}"))?;

    let Some(mut record) = existing else {
        return Ok(());
    };

    let _: Option<ModMetadataRecord> = db
        .delete((TABLE, old_file_name))
        .await
        .map_err(|e| format!("Could not remove old metadata key: {e}"))?;

    record.file_name = new_file_name.to_string();
    let key = record.file_name.clone();
    let _: Option<ModMetadataRecord> = db
        .upsert((TABLE, key.as_str()))
        .content(record)
        .await
        .map_err(|e| format!("Could not save metadata under new file name: {e}"))?;

    Ok(())
}

/// After swapping two `.vpk` files on disk (`file_a` ↔ `file_b`), swap stored metadata so keys
/// still match paths. Handles one-sided metadata (the other mod shows as filename until edited).
pub async fn swap_mod_metadata_records(
    db: &Db,
    file_a: &str,
    file_b: &str,
) -> Result<(), String> {
    if file_a == file_b {
        return Ok(());
    }

    let ra: Option<ModMetadataRecord> = db
        .select((TABLE, file_a))
        .await
        .map_err(|e| format!("Could not read metadata {file_a}: {e}"))?;
    let rb: Option<ModMetadataRecord> = db
        .select((TABLE, file_b))
        .await
        .map_err(|e| format!("Could not read metadata {file_b}: {e}"))?;

    match (ra, rb) {
        (Some(mut a), Some(mut b)) => {
            std::mem::swap(&mut a, &mut b);
            a.file_name = file_a.to_string();
            b.file_name = file_b.to_string();

            let _: Option<ModMetadataRecord> = db
                .delete((TABLE, file_a))
                .await
                .map_err(|e| format!("Could not delete metadata {file_a}: {e}"))?;
            let _: Option<ModMetadataRecord> = db
                .delete((TABLE, file_b))
                .await
                .map_err(|e| format!("Could not delete metadata {file_b}: {e}"))?;

            let _: Option<ModMetadataRecord> = db
                .upsert((TABLE, file_a))
                .content(a)
                .await
                .map_err(|e| format!("Could not save metadata {file_a}: {e}"))?;
            let _: Option<ModMetadataRecord> = db
                .upsert((TABLE, file_b))
                .content(b)
                .await
                .map_err(|e| format!("Could not save metadata {file_b}: {e}"))?;
        }
        (Some(mut a), None) => {
            let _: Option<ModMetadataRecord> = db
                .delete((TABLE, file_a))
                .await
                .map_err(|e| format!("Could not delete metadata {file_a}: {e}"))?;
            a.file_name = file_b.to_string();
            let _: Option<ModMetadataRecord> = db
                .upsert((TABLE, file_b))
                .content(a)
                .await
                .map_err(|e| format!("Could not save metadata {file_b}: {e}"))?;
        }
        (None, Some(mut b)) => {
            let _: Option<ModMetadataRecord> = db
                .delete((TABLE, file_b))
                .await
                .map_err(|e| format!("Could not delete metadata {file_b}: {e}"))?;
            b.file_name = file_a.to_string();
            let _: Option<ModMetadataRecord> = db
                .upsert((TABLE, file_a))
                .content(b)
                .await
                .map_err(|e| format!("Could not save metadata {file_a}: {e}"))?;
        }
        (None, None) => {}
    }

    Ok(())
}
