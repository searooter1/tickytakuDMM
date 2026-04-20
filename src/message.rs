//! All application events (Iced messages): everything the UI or shell can report
//! into `update` lives in [`Message`].

#[derive(Debug, Clone)]
pub enum Message {
    // —— Mod list screen ——
    ModListStartUpload,
    ModListRefresh,
    ModListEnableMod(usize),
    ModListDisableMod(usize),
    ModListRemoveMod(usize),
    ModListEditMod(usize),

    // —— Add / edit mod (shared form) ——
    ModDetailsTitleChanged(String),
    ModDetailsDescriptionChanged(String),
    ModDetailsPickThumbnail,
    ModDetailsClearThumbnail,
    ModDetailsSave,
    ModDetailsCancel,
}
