use directories::ProjectDirs;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::app_db::{self, Db, ModMetadataRecord};
use crate::gameinfo_gi;
use crate::mod_file::ModFile;

#[derive(Debug)]
pub struct ModManager {
    pub mods: Vec<ModFile>,
    surreal: Option<Db>,
    db_runtime: tokio::runtime::Runtime,
}

impl Default for ModManager {
    fn default() -> Self {
        Self::new()
    }
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
        let db_runtime = tokio::runtime::Runtime::new().unwrap_or_else(|e| {
            panic!("Could not start async runtime for SurrealDB: {e}");
        });

        let surreal = match Self::open_surreal(&db_runtime) {
            Ok(db) => Some(db),
            Err(error) => {
                eprintln!("SurrealDB init failed (mod titles/descriptions may be missing): {error}");
                None
            }
        };

        let mut manager = Self {
            mods: Vec::new(),
            surreal,
            db_runtime,
        };

        if let Err(error) = manager.refresh() {
            eprintln!("Failed to load mods on startup: {error}");
        }

        manager
    }

    fn open_surreal(db_runtime: &tokio::runtime::Runtime) -> Result<Db, String> {
        let base = Self::base_data_dir()?;
        Self::ensure_directories()?;
        let legacy_json = base.join("mod_metadata.json");

        db_runtime.block_on(async {
            let db = app_db::open(&base)
                .await
                .map_err(|e| format!("Could not open embedded SurrealDB: {e}"))?;
            app_db::migrate_from_json_if_needed(&db, &legacy_json).await?;
            Ok(db)
        })
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

    fn ensure_directories() -> Result<(), String> {
        fs::create_dir_all(Self::mods_dir()?)
            .map_err(|e| format!("Could not create mods folder: {e}"))?;

        fs::create_dir_all(Self::thumbnails_dir()?)
            .map_err(|e| format!("Could not create thumbnails folder: {e}"))?;

        Ok(())
    }

    fn load_metadata_map(&self) -> Result<HashMap<String, ModMetadataRecord>, String> {
        let Some(ref db) = self.surreal else {
            return Ok(HashMap::new());
        };

        let entries = self
            .db_runtime
            .block_on(app_db::load_all_metadata(db))?;

        let mut map = HashMap::new();
        for entry in entries {
            map.insert(entry.file_name.clone(), entry);
        }

        Ok(map)
    }

    fn save_metadata_entry(&self, record: ModMetadataRecord) -> Result<(), String> {
        let Some(ref db) = self.surreal else {
            return Err(String::from(
                "Embedded database is unavailable; cannot save mod metadata.",
            ));
        };

        self.db_runtime
            .block_on(app_db::upsert_metadata(db, record))
    }

    fn delete_metadata_entry(&self, file_name: &str) -> Result<(), String> {
        let Some(ref db) = self.surreal else {
            return Err(String::from(
                "Embedded database is unavailable; cannot update mod metadata.",
            ));
        };

        self.db_runtime
            .block_on(app_db::delete_metadata(db, file_name))
    }

    pub fn refresh(&mut self) -> Result<(), String> {
        Self::ensure_directories()?;

        let mods_dir = Self::mods_dir()?;
        Self::migrate_legacy_vpks(&mods_dir, &self.surreal, &self.db_runtime)?;

        let metadata_map = self.load_metadata_map()?;

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
                        thumbnail_path: metadata
                            .thumbnail_path
                            .as_ref()
                            .map(PathBuf::from),
                    });
                } else {
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
                Self::parse_pak_dir_slot(&a.file_name),
                Self::parse_pak_dir_slot(&b.file_name),
            ) {
                (Some(a_index), Some(b_index)) => a_index.cmp(&b_index),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()),
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
        let destination = Self::next_pak_destination(&mods_dir)?;

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

        self.save_metadata_entry(ModMetadataRecord {
            metadata_id: Uuid::new_v4(),
            file_name: file_name.clone(),
            title,
            description,
            thumbnail_path: saved_thumbnail_path
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned()),
        })?;
        self.refresh()?;

        Ok(destination)
    }

    pub fn remove_mod(&mut self, index: usize) -> Result<(), String> {
        let (file_name, file_path, thumbnail_path) = {
            let Some(mod_file) = self.mods.get(index) else {
                return Err(String::from("Invalid mod index"));
            };
            (
                mod_file.file_name.clone(),
                mod_file.path.clone(),
                mod_file.thumbnail_path.clone(),
            )
        };

        // Clear the Deadlock addons copy first so the game does not keep loading a removed mod.
        self.disable_mod(index)?;

        fs::remove_file(&file_path)
            .map_err(|e| format!("Could not remove mod file: {e}"))?;

        if let Some(thumbnail_path) = thumbnail_path {
            if thumbnail_path.exists() {
                let _ = fs::remove_file(thumbnail_path);
            }
        }

        self.delete_metadata_entry(&file_name)?;

        self.refresh()?;
        Ok(())
    }

    pub fn enable_mod(&mut self, index: usize) -> Result<PathBuf, String> {
        let Some(mod_file) = self.mods.get(index) else {
            return Err(String::from("Invalid mod index"));
        };

        let gameinfo = Self::deadlock_gameinfo_path()?;
        gameinfo_gi::ensure_addon_search_paths(&gameinfo)?;

        let source_path = mod_file.path.clone();
        let file_name = mod_file.file_name.clone();
        let addons_dir = Self::resolve_deadlock_addons_dir()?;

        fs::create_dir_all(&addons_dir)
            .map_err(|e| format!("Could not create Deadlock addons folder: {e}"))?;

        let destination = addons_dir.join(file_name);

        fs::copy(&source_path, &destination)
            .map_err(|e| format!("Could not copy .vpk into Deadlock addons: {e}"))?;

        Ok(destination)
    }

    pub fn disable_mod(&mut self, index: usize) -> Result<(), String> {
        let Some(mod_file) = self.mods.get(index) else {
            return Err(String::from("Invalid mod index"));
        };

        let file_name = mod_file.file_name.clone();
        let addons_dir = Self::resolve_deadlock_addons_dir()?;
        let destination = addons_dir.join(file_name);

        if destination.exists() {
            fs::remove_file(&destination)
                .map_err(|e| format!("Could not remove .vpk from Deadlock addons: {e}"))?;
        }

        Ok(())
    }

    /// Whether this mod's `.vpk` is currently present under the Deadlock `addons` folder.
    pub fn is_mod_enabled(&self, index: usize) -> bool {
        let Some(mod_file) = self.mods.get(index) else {
            return false;
        };

        let Ok(addons_dir) = Self::resolve_deadlock_addons_dir() else {
            return false;
        };

        addons_dir.join(&mod_file.file_name).is_file()
    }

    pub fn update_mod_entry(
        &mut self,
        index: usize,
        title: String,
        description: Option<String>,
        thumbnail_path: Option<PathBuf>,
        original_thumbnail_path: Option<PathBuf>,
    ) -> Result<(), String> {
        let Some(mod_file) = self.mods.get(index) else {
            return Err(String::from("Invalid mod index"));
        };

        let file_name = mod_file.file_name.clone();
        Self::ensure_directories()?;

        let final_thumbnail = if thumbnail_path == original_thumbnail_path {
            thumbnail_path.clone()
        } else if thumbnail_path.is_none() {
            if let Some(ref old) = original_thumbnail_path {
                if old.exists() {
                    let _ = fs::remove_file(old);
                }
            }
            None
        } else {
            let selected = thumbnail_path.as_ref().unwrap();

            if !Self::is_supported_thumbnail_file(selected) {
                return Err(String::from(
                    "Thumbnail must be a .png, .jpg, .jpeg, or .webp file",
                ));
            }

            let new_path = Self::copy_thumbnail(selected)?;

            if let Some(ref old) = original_thumbnail_path {
                if old != &new_path && old.exists() {
                    let _ = fs::remove_file(old);
                }
            }

            Some(new_path)
        };

        let metadata_id = self
            .load_metadata_map()?
            .get(&file_name)
            .map(|entry| entry.metadata_id)
            .unwrap_or_else(Uuid::new_v4);

        self.save_metadata_entry(ModMetadataRecord {
            metadata_id,
            file_name: file_name.clone(),
            title,
            description,
            thumbnail_path: final_thumbnail
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned()),
        })?;
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

    /// Next free `pakNN_dir.vpk` slot (`NN` in `01`..=`99`). Lower `NN` = higher load priority.
    fn next_pak_destination(mods_dir: &Path) -> Result<PathBuf, String> {
        let mut used = HashSet::new();

        if let Ok(entries) = fs::read_dir(mods_dir) {
            for entry in entries.flatten() {
                let file_name = entry.file_name();
                let file_name = file_name.to_string_lossy();
                if let Some(slot) = Self::parse_pak_dir_slot(&file_name) {
                    used.insert(slot);
                }
            }
        }

        for slot in 1..=99u32 {
            if !used.contains(&slot) {
                return Ok(mods_dir.join(format!("pak{slot:02}_dir.vpk")));
            }
        }

        Err(String::from(
            "All 99 mod slots (pak01_dir … pak99_dir) are in use. Remove a mod before adding another.",
        ))
    }

    /// Parses `pakNN_dir.vpk` → `NN` (1–99).
    fn parse_pak_dir_slot(file_name: &str) -> Option<u32> {
        let lower = file_name.to_ascii_lowercase();
        if !lower.starts_with("pak") || !lower.ends_with("_dir.vpk") {
            return None;
        }
        let mid = lower.strip_prefix("pak")?.strip_suffix("_dir.vpk")?;
        if mid.len() != 2 || !mid.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        let n: u32 = mid.parse().ok()?;
        if (1..=99).contains(&n) {
            Some(n)
        } else {
            None
        }
    }

    fn parse_legacy_pak01_slot(file_name: &str) -> Option<u32> {
        let lower = file_name.to_ascii_lowercase();
        let index_text = lower
            .strip_prefix("pak01_")?
            .strip_suffix(".vpk")?;

        if index_text.len() != 3 || !index_text.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }

        index_text.parse::<u32>().ok()
    }

    fn migrate_legacy_vpks(
        mods_dir: &Path,
        surreal: &Option<Db>,
        db_runtime: &tokio::runtime::Runtime,
    ) -> Result<(), String> {
        let mut used_slots = HashSet::new();
        let mut legacy = Vec::new();

        let entries = fs::read_dir(mods_dir)
            .map_err(|e| format!("Could not read mods folder for migration: {e}"))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Could not read mods folder entry: {e}"))?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if !Self::is_vpk_file(&path) {
                continue;
            }
            if let Some(slot) = Self::parse_pak_dir_slot(&name) {
                used_slots.insert(slot);
            } else if let Some(old_idx) = Self::parse_legacy_pak01_slot(&name) {
                legacy.push((old_idx, path, name));
            }
        }

        legacy.sort_by_key(|(idx, _, _)| *idx);

        for (_, path, old_name) in legacy {
            let slot = (1..=99u32)
                .find(|s| !used_slots.contains(s))
                .ok_or_else(|| {
                    String::from(
                        "Too many mods to migrate legacy pak01_NNN.vpk names; free a slot under 99.",
                    )
                })?;
            used_slots.insert(slot);

            let new_name = format!("pak{slot:02}_dir.vpk");
            let new_path = mods_dir.join(&new_name);
            if new_path.exists() {
                return Err(format!(
                    "Migration blocked: {} already exists",
                    new_path.display()
                ));
            }

            fs::rename(&path, &new_path)
                .map_err(|e| format!("Could not rename {} → {}: {e}", old_name, new_name))?;

            if let Some(db) = surreal {
                db_runtime.block_on(app_db::rename_metadata_file_name(
                    db,
                    &old_name,
                    &new_name,
                ))?;
            }

            Self::rename_addon_vpk_if_present(&old_name, &new_name)?;
        }

        Ok(())
    }

    /// Swap two `.vpk` files in `addons` named `a` and `b` (if present), using a temp file.
    fn swap_addon_vpks_named(a: &str, b: &str) -> Result<(), String> {
        let Ok(addons_dir) = Self::resolve_deadlock_addons_dir() else {
            return Ok(());
        };

        let pa = addons_dir.join(a);
        let pb = addons_dir.join(b);
        let exists_a = pa.is_file();
        let exists_b = pb.is_file();

        if exists_a && exists_b {
            let tmp = addons_dir.join(format!(".swap_addon_{}.vpk", Uuid::new_v4()));
            fs::rename(&pa, &tmp).map_err(|e| format!("Could not stage addon swap: {e}"))?;
            fs::rename(&pb, &pa).map_err(|e| format!("Could not swap addon files: {e}"))?;
            fs::rename(&tmp, &pb).map_err(|e| format!("Could not finish addon swap: {e}"))?;
        } else if exists_a && !exists_b {
            fs::rename(&pa, &pb).map_err(|e| format!("Could not rename addon {}: {e}", pa.display()))?;
        } else if exists_b && !exists_a {
            fs::rename(&pb, &pa).map_err(|e| format!("Could not rename addon {}: {e}", pb.display()))?;
        }

        Ok(())
    }

    /// Swap the mod at `upper_index` with the one below it (`upper_index + 1`). List order is
    /// higher priority first; moving **up** decreases index.
    fn swap_adjacent_mods(&mut self, upper_index: usize) -> Result<(), String> {
        let lower_index = upper_index + 1;
        if lower_index >= self.mods.len() {
            return Ok(());
        }

        let na = self.mods[upper_index].file_name.clone();
        let nb = self.mods[lower_index].file_name.clone();
        let pa = self.mods[upper_index].path.clone();
        let pb = self.mods[lower_index].path.clone();

        if na == nb {
            return Ok(());
        }

        let mods_dir = Self::mods_dir()?;
        let tmp = mods_dir.join(format!(".swap_{}.vpk", Uuid::new_v4()));

        fs::rename(&pa, &tmp).map_err(|e| format!("Could not stage library swap: {e}"))?;
        fs::rename(&pb, &pa).map_err(|e| format!("Could not swap mod files: {e}"))?;
        fs::rename(&tmp, &pb).map_err(|e| format!("Could not finish library swap: {e}"))?;

        Self::swap_addon_vpks_named(&na, &nb)?;

        if let Some(db) = &self.surreal {
            self.db_runtime
                .block_on(app_db::swap_mod_metadata_records(db, &na, &nb))?;
        }

        self.refresh()?;
        Ok(())
    }

    /// Move mod at `index` one step higher in load order (toward `pak01_dir`).
    pub fn move_mod_up(&mut self, index: usize) -> Result<(), String> {
        if index == 0 || index >= self.mods.len() {
            return Ok(());
        }
        self.swap_adjacent_mods(index - 1)
    }

    /// Move mod at `index` one step lower in load order (toward `pak99_dir`).
    pub fn move_mod_down(&mut self, index: usize) -> Result<(), String> {
        if index + 1 >= self.mods.len() {
            return Ok(());
        }
        self.swap_adjacent_mods(index)
    }

    /// If a copy exists under Deadlock `addons`, rename it to match the library `.vpk` name.
    fn rename_addon_vpk_if_present(old_file_name: &str, new_file_name: &str) -> Result<(), String> {
        if old_file_name == new_file_name {
            return Ok(());
        }

        let Ok(addons_dir) = Self::resolve_deadlock_addons_dir() else {
            return Ok(());
        };

        let old_p = addons_dir.join(old_file_name);
        let new_p = addons_dir.join(new_file_name);
        if old_p.is_file() {
            if new_p.exists() {
                fs::remove_file(&new_p).map_err(|e| {
                    format!(
                        "Could not replace {} in Deadlock addons: {e}",
                        new_p.display()
                    )
                })?;
            }
            fs::rename(&old_p, &new_p).map_err(|e| {
                format!(
                    "Could not rename addon {} → {}: {e}",
                    old_p.display(),
                    new_p.display()
                )
            })?;
        }

        Ok(())
    }

    fn resolve_deadlock_addons_dir() -> Result<PathBuf, String> {
        if let Some(install_dir) = Self::find_deadlock_install_dir() {
            return Ok(install_dir.join("game").join("citadel").join("addons"));
        }

        Err(String::from(
            "Could not find Deadlock install folder under any Steam library. Make sure Deadlock is installed in a Steam library.",
        ))
    }

    /// `…/Deadlock/game/citadel/gameinfo.gi`
    fn deadlock_gameinfo_path() -> Result<PathBuf, String> {
        let Some(install_dir) = Self::find_deadlock_install_dir() else {
            return Err(String::from(
                "Could not find Deadlock install folder under any Steam library. Make sure Deadlock is installed in a Steam library.",
            ));
        };

        let path = install_dir
            .join("game")
            .join("citadel")
            .join("gameinfo.gi");
        if !path.is_file() {
            return Err(format!(
                "Deadlock gameinfo not found at {} (install may be incomplete).",
                path.display()
            ));
        }

        Ok(path)
    }

    fn find_deadlock_install_dir() -> Option<PathBuf> {
        for candidate in Self::deadlock_install_candidates() {
            if candidate.exists() {
                return Some(candidate);
            }
        }

        None
    }

    fn deadlock_install_candidates() -> Vec<PathBuf> {
        let mut roots = Vec::new();

        if let Ok(program_files_x86) = env::var("ProgramFiles(x86)") {
            roots.push(PathBuf::from(program_files_x86).join("Steam"));
        }

        if let Ok(program_files) = env::var("ProgramFiles") {
            roots.push(PathBuf::from(program_files).join("Steam"));
        }

        for drive in 'A'..='Z' {
            roots.push(PathBuf::from(format!("{drive}:\\Steam")));
            roots.push(PathBuf::from(format!("{drive}:\\SteamLibrary")));
        }

        let mut candidates = Vec::new();
        for root in roots {
            candidates.push(
                root.join("steamapps")
                    .join("common")
                    .join("Deadlock"),
            );
        }

        candidates
    }
}