use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ModFile {
    // The real filename used by the game.
    // Example: pak01_000.vpk
    pub file_name: String,

    // Full path to the mod file on disk.
    pub path: PathBuf,

    // User-facing metadata stored by the app.
    pub title: String,
    pub description: Option<String>,
    pub thumbnail_path: Option<PathBuf>,
}