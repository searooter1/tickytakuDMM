use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ModFile {
    // The actual file name stored on disk for the game to use.
    // Example: pak01_000.vpk
    pub file_name: String,

    // Full path to the .vpk file on disk
    pub path: PathBuf,

    // User-facing metadata
    pub title: String,
    pub description: Option<String>,
    pub thumbnail_path: Option<PathBuf>,
}