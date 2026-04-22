//! All application events (Iced messages): everything the UI or shell can report
//! into `update` lives in [`Message`].

use crate::gamebanana::{FileEntry, ImportPayload, ModSummary};
use crate::state::GameBananaBrowseCategory;

#[derive(Debug, Clone)]
pub enum Message {
    // —— Mod list screen ——
    ModListStartUpload,
    ModListRefresh,
    ModListOpenGameBanana,
    ModListEnableMod(usize),
    ModListDisableMod(usize),
    ModListRemoveMod(usize),
    ModListEditMod(usize),
    ModListMoveModUp(usize),
    ModListMoveModDown(usize),

    // —— Add / edit mod (shared form) ——
    ModDetailsTitleChanged(String),
    ModDetailsDescriptionChanged(String),
    ModDetailsPickThumbnail,
    ModDetailsClearThumbnail,
    ModDetailsSave,
    ModDetailsCancel,

    // —— GameBanana browser ——
    GameBananaBack,
    GameBananaBrowseCategorySelected(GameBananaBrowseCategory),
    GameBananaSearchInput(String),
    GameBananaSearchSubmit,
    GameBananaBrowseMode,
    GameBananaPagePrev,
    GameBananaPageNext,
    GameBananaSelectMod(u64),
    /// Cycle preview images for a mod (`next`: true = forward, false = back).
    GameBananaThumbCarousel { mod_id: u64, next: bool },
    GameBananaOpenModUrl(String),
    GameBananaDownloadFile(FileEntry),
    GameBananaListLoaded {
        generation: u64,
        page: u32,
        result: Result<(Vec<ModSummary>, u64), String>,
    },
    GameBananaThumbnailsReady {
        list_generation: u64,
        loaded: Vec<(u64, Vec<Vec<u8>>)>,
    },
    GameBananaFilesLoaded {
        generation: u64,
        mod_id: u64,
        result: Result<(Vec<FileEntry>, String), String>,
    },
    GameBananaImportDone(Result<ImportPayload, String>),
}
