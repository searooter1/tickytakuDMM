use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::mod_file::ModFile;

#[derive(Debug, Default)]
pub struct ModManager {
    pub mods: Vec<ModFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModMetadata {
    // Stable internal ID for metadata tracking
    id: Uuid,

    // Actual stored filename on disk, like pak01_000.vpk
    file_name: String,

    // User-facing fields
    title: String,
    description: Option<String>,
    thumbnail_path: Option<PathBuf>,
}

impl ModManager {
    fn is_vpk_file(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("vpk"))
    }

    fn is_supported_thumbnail_file(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| {
                ext.eq_ignore_ascii_case("png")
                    || ext.eq_ignore_ascii_case("jpg")
                    || ext.eq_ignore_ascii_case("jpeg")
                    || ext.eq_ignore_ascii_case("webp")
            })
    }

    pub fn new() -> Self {
        let mut manager = Self::default();

        if let Err(error) = manager.refresh() {
            eprintln!("Failed to load mods on startup: {error}");
        }

        manager
    }

    pub fn base_data_dir() -> Result<PathBuf, String> {
        let project_dirs = ProjectDirs::from("com", "tickytaku", "ticktakuDMM")
            .ok_or_else(|| String::from("Could not determine app data directory"))?;

        Ok(project_dirs.data_local_dir().to_path_buf())
    }

    pub fn mods_dir() -> Result<PathBuf, String> {
        Ok(Self::base_data_dir()?.join("mods"))
    }

    pub fn thumbnails_dir() -> Result<PathBuf, String> {
        Ok(Self::base_data_dir()?.join("thumbnails"))
    }

    fn metadata_path() -> Result<PathBuf, String> {
        Ok(Self::base_data_dir()?.join("mod_metadata.json"))
    }

    fn ensure_directories() -> Result<(), String> {
        fs::create_dir_all(Self::mods_dir()?)
            .map_err(|e| format!("Could not create mods folder: {e}"))?;

        fs::create_dir_all(Self::thumbnails_dir()?)
            .map_err(|e| format!("Could not create thumbnails folder: {e}"))?;

        Ok(())
    }

    fn load_metadata_map() -> Result<HashMap<String, ModMetadata>, String> {
        let metadata_path = Self::metadata_path()?;

        if !metadata_path.exists() {
            return Ok(HashMap::new());
        }

        let text = fs::read_to_string(&metadata_path)
            .map_err(|e| format!("Could not read metadata file: {e}"))?;

        let entries: Vec<ModMetadata> = serde_json::from_str(&text)
            .map_err(|e| format!("Could not parse metadata file: {e}"))?;

        let mut map = HashMap::new();
        for entry in entries {
            map.insert(entry.file_name.clone(), entry);
        }

        Ok(map)
    }

    fn save_metadata_map(map: &HashMap<String, ModMetadata>) -> Result<(), String> {
        let mut entries: Vec<ModMetadata> = map.values().cloned().collect();

        entries.sort_by(|a, b| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()));

        let json = serde_json::to_string_pretty(&entries)
            .map_err(|e| format!("Could not serialize metadata: {e}"))?;

        fs::write(Self::metadata_path()?, json)
            .map_err(|e| format!("Could not write metadata file: {e}"))?;

        Ok(())
    }

    pub fn refresh(&mut self) -> Result<(), String> {
        Self::ensure_directories()?;

        let mods_dir = Self::mods_dir()?;
        let metadata_map = Self::load_metadata_map()?;

        let entries = fs::read_dir(&mods_dir)
            .map_err(|e| format!("Could not read mods folder: {e}"))?;

        let mut found_mods = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|e| format!("Could not read folder entry: {e}"))?;
            let path = entry.path();

            if path.is_file() {
                let file_name = entry.file_name().to_string_lossy().to_string();

                if !Self::is_vpk_file(&path) {
                    continue;
                }

                if let Some(metadata) = metadata_map.get(&file_name) {
                    found_mods.push(ModFile {
                        file_name: file_name.clone(),
                        path,
                        title: metadata.title.clone(),
                        description: metadata.description.clone(),
                        thumbnail_path: metadata.thumbnail_path.clone(),
                    });
                } else {
                    // Fallback if metadata is missing
                    found_mods.push(ModFile {
                        file_name: file_name.clone(),
                        path,
                        title: file_name.clone(),
                        description: None,
                        thumbnail_path: None,
                    });
                }
            }
        }

        found_mods.sort_by(|a, b| {
            match (
                Self::parse_pak_index(&a.file_name),
                Self::parse_pak_index(&b.file_name),
            ) {
                (Some(a_index), Some(b_index)) => a_index.cmp(&b_index),
                _ => a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()),
            }
        });

        self.mods = found_mods;
        Ok(())
    }

    pub fn import_file_with_metadata(
        &mut self,
        source: &Path,
        title: String,
        description: Option<String>,
        thumbnail_source: Option<&Path>,
    ) -> Result<PathBuf, String> {
        if !Self::is_vpk_file(source) {
            return Err(String::from("Only .vpk files are supported"));
        }

        Self::ensure_directories()?;

        let mods_dir = Self::mods_dir()?;
        let destination = Self::next_pak_destination(&mods_dir);

        fs::copy(source, &destination)
            .map_err(|e| format!("Could not copy mod file: {e}"))?;

        let file_name = destination
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| String::from("Saved mod file has invalid name"))?
            .to_string();

        let saved_thumbnail_path = if let Some(thumbnail_source) = thumbnail_source {
            if !Self::is_supported_thumbnail_file(thumbnail_source) {
                return Err(String::from(
                    "Thumbnail must be a .png, .jpg, .jpeg, or .webp file",
                ));
            }

            Some(Self::copy_thumbnail(thumbnail_source)?)
        } else {
            None
        };

        let mut metadata_map = Self::load_metadata_map()?;

        metadata_map.insert(
            file_name.clone(),
            ModMetadata {
                id: Uuid::new_v4(),
                file_name: file_name.clone(),
                title,
                description,
                thumbnail_path: saved_thumbnail_path,
            },
        );

        Self::save_metadata_map(&metadata_map)?;
        self.refresh()?;

        Ok(destination)
    }

    pub fn remove_mod(&mut self, index: usize) -> Result<(), String> {
        let Some(mod_file) = self.mods.get(index) else {
            return Err(String::from("Invalid mod index"));
        };

        let file_name = mod_file.file_name.clone();
        let file_path = mod_file.path.clone();
        let thumbnail_path = mod_file.thumbnail_path.clone();

        fs::remove_file(&file_path)
            .map_err(|e| format!("Could not remove mod file: {e}"))?;

        if let Some(thumbnail_path) = thumbnail_path {
            if thumbnail_path.exists() {
                let _ = fs::remove_file(thumbnail_path);
            }
        }

        let mut metadata_map = Self::load_metadata_map()?;
        metadata_map.remove(&file_name);
        Self::save_metadata_map(&metadata_map)?;

        self.refresh()?;
        Ok(())
    }

    fn copy_thumbnail(source: &Path) -> Result<PathBuf, String> {
        let thumbnails_dir = Self::thumbnails_dir()?;

        let extension = source
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("png");

        let file_name = format!("{}.{}", Uuid::new_v4(), extension.to_lowercase());
        let destination = thumbnails_dir.join(file_name);

        fs::copy(source, &destination)
            .map_err(|e| format!("Could not copy thumbnail: {e}"))?;

        Ok(destination)
    }

    fn next_pak_destination(mods_dir: &Path) -> PathBuf {
        let mut used_indexes = Vec::new();

        if let Ok(entries) = fs::read_dir(mods_dir) {
            for entry in entries.flatten() {
                let file_name = entry.file_name();
                let file_name = file_name.to_string_lossy();

                if let Some(index) = Self::parse_pak_index(&file_name) {
                    used_indexes.push(index);
                }
            }
        }

        used_indexes.sort_unstable();
        used_indexes.dedup();

        let mut next_index = 0;
        for used_index in used_indexes {
            if used_index == next_index {
                next_index += 1;
            } else if used_index > next_index {
                break;
            }
        }

        mods_dir.join(format!("pak01_{next_index:03}.vpk"))
    }

    fn parse_pak_index(file_name: &str) -> Option<u32> {
        let lower = file_name.to_ascii_lowercase();
        let index_text = lower
            .strip_prefix("pak01_")?
            .strip_suffix(".vpk")?;

        if index_text.len() != 3 || !index_text.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }

        index_text.parse::<u32>().ok()
    }
}