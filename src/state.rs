use std::collections::HashMap;
use std::path::PathBuf;

use iced::widget::image::Handle;

use crate::gamebanana::{FileEntry, ModSummary};

#[derive(Debug)]
pub struct AppState {
    pub status: String,
    pub page: Page,
}

#[derive(Debug)]
pub enum Page {
    ModList(ModListState),
    ModDetails(ModDetailsState),
    GameBanana(GameBananaState),
}

#[derive(Debug, Default, Clone)]
pub struct ModListState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameBananaListSource {
    Browse,
    /// Last submitted search string (≥ 2 chars).
    Search(String),
}

/// GameBanana `Generic_Category` filter for Deadlock (`gameid` 20948).  
/// IDs match `gamebanana.com/mods/cats/…` section roots (models/skins, SFX, HUD).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GameBananaBrowseCategory {
    /// No category filter — all mod types for this game.
    #[default]
    All,
    /// Model replacement / skins (`…/mods/cats/33154`).
    ModelsAndSkins,
    /// Sound effects (`…/mods/cats/39389`).
    SoundEffects,
    /// HUD, crosshairs, and many UI / icon packs (`…/mods/cats/31713`).
    HudUiAndIcons,
}

impl GameBananaBrowseCategory {
    pub const VARIANTS: &'static [Self] = &[
        Self::All,
        Self::ModelsAndSkins,
        Self::SoundEffects,
        Self::HudUiAndIcons,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::All => "All types",
            Self::ModelsAndSkins => "Models & skins",
            Self::SoundEffects => "Sound effects",
            Self::HudUiAndIcons => "HUD & icons",
        }
    }

    pub const fn category_id(self) -> Option<u32> {
        match self {
            Self::All => None,
            Self::ModelsAndSkins => Some(33154),
            Self::SoundEffects => Some(39389),
            Self::HudUiAndIcons => Some(31713),
        }
    }
}

#[derive(Clone)]
pub struct GameBananaState {
    pub source: GameBananaListSource,
    /// Limits browse + search to a Deadlock mod section (skins, SFX, HUD, etc.).
    pub browse_category: GameBananaBrowseCategory,
    pub search_draft: String,
    pub page: u32,
    pub per_page: u32,
    pub total_count: u64,
    pub mods: Vec<ModSummary>,
    pub list_loading: bool,
    pub list_error: Option<String>,
    /// Incremented whenever a list fetch starts; stale responses are dropped.
    pub list_request_generation: u64,

    /// Preview images for the current list page, in carousel order (`iced` handles).
    pub thumbnails: HashMap<u64, Vec<Handle>>,
    /// Current slide index per mod for the preview carousel.
    pub thumb_carousel_index: HashMap<u64, usize>,

    pub selected_mod_id: Option<u64>,
    pub selected_mod_name: Option<String>,
    pub selected_preview_url: Option<String>,
    pub files: Vec<FileEntry>,
    pub files_loading: bool,
    pub files_error: Option<String>,
    pub files_request_generation: u64,

    pub import_busy: bool,
}

impl std::fmt::Debug for GameBananaState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GameBananaState")
            .field("page", &self.page)
            .field("mods", &self.mods.len())
            .field("thumbnails", &self.thumbnails.len())
            .field("thumb_carousel_index", &self.thumb_carousel_index.len())
            .field("list_request_generation", &self.list_request_generation)
            .field("import_busy", &self.import_busy)
            .finish_non_exhaustive()
    }
}

impl GameBananaState {
    pub fn new_browse() -> Self {
        Self {
            source: GameBananaListSource::Browse,
            browse_category: GameBananaBrowseCategory::default(),
            search_draft: String::new(),
            page: 1,
            per_page: 20,
            total_count: 0,
            mods: Vec::new(),
            list_loading: true,
            list_error: None,
            list_request_generation: 1,
            thumbnails: HashMap::new(),
            thumb_carousel_index: HashMap::new(),
            selected_mod_id: None,
            selected_mod_name: None,
            selected_preview_url: None,
            files: Vec::new(),
            files_loading: false,
            files_error: None,
            files_request_generation: 0,
            import_busy: false,
        }
    }
}

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
