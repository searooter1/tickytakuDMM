use std::path::PathBuf;

#[derive(Debug)]
pub struct AppState {
    pub status: String,
    pub page: Page,
}

#[derive(Debug)]
pub enum Page {
    ModList(ModListState),
    ImportMod(ImportModState),
}

#[derive(Debug, Default, Clone)]
pub struct ModListState;

#[derive(Debug, Clone)]
pub struct ImportModState {
    pub mod_path: PathBuf,
    pub title: String,
    pub description: String,
    pub thumbnail_path: Option<PathBuf>,
}

impl ImportModState {
    pub fn new(mod_path: PathBuf) -> Self {
        let title = mod_path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("New Mod")
            .to_string();

        Self {
            mod_path,
            title,
            description: String::new(),
            thumbnail_path: None,
        }
    }

    pub fn trimmed_title(&self) -> String {
        self.title.trim().to_string()
    }

    pub fn trimmed_description(&self) -> Option<String> {
        let trimmed = self.description.trim();

        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }
}