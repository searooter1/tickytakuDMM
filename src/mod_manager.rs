use directories::ProjectDirs;
use std::fs;
use std::path::{Path, PathBuf};

use crate::mod_file::ModFile;

#[derive(Debug, Default)]
pub struct ModManager {
    pub mods: Vec<ModFile>,
}

impl ModManager {
    fn is_vpk_file(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("vpk"))
    }

    // Constructor, creates an instance of mod manager
    pub fn new() -> Self {
        let mut manager = Self::default();

        // Gets mods files in mod folder via refresh, if fail then error
        if let Err(error) = manager.refresh() {
            eprintln!("Failed to load mods on startup: {error}");
        }

        manager
    }

    pub fn mods_dir() -> Result<PathBuf, String> {
        let project_dirs = ProjectDirs::from("com", "tickytaku", "ticktakuDMM")
            .ok_or_else(|| String::from("Could not determine app data directory"))?;

        Ok(project_dirs.data_local_dir().join("mods"))
    }

    pub fn refresh(&mut self) -> Result<(), String> {
        let mods_dir = Self::mods_dir()?;

        fs::create_dir_all(&mods_dir)
            .map_err(|e| format!("Could not create mods folder: {e}"))?;

        let mut found_mods = Vec::new();

        let entries = fs::read_dir(&mods_dir)
            .map_err(|e| format!("Could not read mods folder: {e}"))?;

        // Iterates over each entry in the mod files and puts them in the mods vector
        for entry in entries {
            let entry = entry.map_err(|e| format!("Could not read folder entry: {e}"))?;
            let path = entry.path();

            if path.is_file() {
                let file_name = entry.file_name().to_string_lossy().to_string();

                found_mods.push(ModFile { file_name, path });
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

    // Add file to mod folder
    pub fn import_file(&mut self, source: &Path) -> Result<PathBuf, String> {
        if !Self::is_vpk_file(source) {
            return Err(String::from("Only .vpk files are supported"));
        }

        let mods_dir = Self::mods_dir()?;

        fs::create_dir_all(&mods_dir)
            .map_err(|e| format!("Could not create mods folder: {e}"))?;

        let destination = Self::next_pak_destination(&mods_dir);

        fs::copy(source, &destination)
            .map_err(|e| format!("Could not copy file: {e}"))?;

        self.refresh()?;
        Ok(destination)
    }

    // removes a mod from the mod folder
    pub fn remove_mod(&mut self, index: usize) -> Result<(), String> {
        let Some(mod_file) = self.mods.get(index) else {
            return Err(String::from("Invalid mod index"));
        };

        fs::remove_file(&mod_file.path)
            .map_err(|e| format!("Could not remove file: {e}"))?;

        self.refresh()?;
        Ok(())
    }

    // Uses Deadlock-style naming: pak01_000.vpk, pak01_001.vpk, ...
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