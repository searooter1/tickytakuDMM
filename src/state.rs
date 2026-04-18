use std::path::PathBuf;

#[derive(Debug)]
pub struct AppState {
    pub status: String,
    pub page: Page,
}

#[derive(Debug)]
pub enum Page {
    ModList(ModListState),
    ModDetails(ModDetailsState),
}

#[derive(Debug, Default, Clone)]
pub struct ModListState;

/// Add a new mod (copy VPK) vs change metadata for an installed mod.
#[derive(Debug, Clone)]
pub enum ModDetailsMode {
    Import {
        source_path: PathBuf,
    },
    Edit {
        mod_index: usize,
        file_name: String,
        original_thumbnail_path: Option<PathBuf>,
    },
}

#[derive(Debug, Clone)]
pub struct ModDetailsState {
    pub mode: ModDetailsMode,
    pub title: String,
    pub description: String,
    pub thumbnail_path: Option<PathBuf>,
}

impl ModDetailsState {
    pub fn import(source_path: PathBuf) -> Self {
        let title = source_path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("New Mod")
            .to_string();

        Self {
            mode: ModDetailsMode::Import { source_path },
            title,
            description: String::new(),
            thumbnail_path: None,
        }
    }

    pub fn edit(
        mod_index: usize,
        file_name: String,
        title: String,
        description: String,
        thumbnail_path: Option<PathBuf>,
    ) -> Self {
        Self {
            mode: ModDetailsMode::Edit {
                mod_index,
                file_name,
                original_thumbnail_path: thumbnail_path.clone(),
            },
            title,
            description,
            thumbnail_path,
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
