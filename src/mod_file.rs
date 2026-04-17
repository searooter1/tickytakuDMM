use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ModFile {
    // Real filename used by the game, for example: pak01_000.vpk
    pub file_name: String,

    // Full path to the real mod file on disk
    pub path: PathBuf,

    // User-facing metadata used by this app
    pub title: String,
    pub description: Option<String>,
    pub thumbnail_path: Option<PathBuf>,
}