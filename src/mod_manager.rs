use directories::ProjectDirs;
use std::fs;
use std::path::{Path, PathBuf};

use crate::mod_file::ModFile;

#[derive(Debug, Default)]
pub struct ModManager {
    pub mods: Vec<ModFile>,
}

impl ModManager {

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

        found_mods.sort_by(|a, b| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()));

        self.mods = found_mods;
        Ok(())
    }

    // Add file to mod folder
    pub fn import_file(&mut self, source: &Path) -> Result<PathBuf, String> {
        let mods_dir = Self::mods_dir()?;

        fs::create_dir_all(&mods_dir)
            .map_err(|e| format!("Could not create mods folder: {e}"))?;

        let file_name = source
            .file_name()
            .ok_or_else(|| String::from("Selected file has no valid name"))?
            .to_string_lossy()
            .to_string();

        let destination = Self::unique_destination(&mods_dir, &file_name);

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

    // Makes sure files in the mod folder have unique names
    fn unique_destination(mods_dir: &Path, file_name: &str) -> PathBuf {
        let initial = mods_dir.join(file_name);

        if !initial.exists() {
            return initial;
        }

        let source_path = Path::new(file_name);
        let stem = source_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("mod");
        let ext = source_path.extension().and_then(|s| s.to_str());

        let mut i = 1;
        loop {
            let candidate = match ext {
                Some(ext) => mods_dir.join(format!("{stem}_{i}.{ext}")),
                None => mods_dir.join(format!("{stem}_{i}")),
            };

            if !candidate.exists() {
                return candidate;
            }

            i += 1;
        }
    }
}