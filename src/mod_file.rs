use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ModFile {
    pub file_name: String,
    pub path: PathBuf,
    pub title: String,
    pub description: Option<String>,
    pub thumbnail_path: Option<PathBuf>,
}