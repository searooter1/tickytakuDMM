//! All application events (Iced messages): everything the UI or shell can report
//! into `update` lives in [`Message`].

#[derive(Debug, Clone)]
pub enum Message {
    // —— Mod list screen ——
    ModListStartUpload,
    ModListRefresh,
    ModListRemoveMod(usize),

    // —— Import mod screen ——
    ImportTitleChanged(String),
    ImportDescriptionChanged(String),
    ImportPickThumbnail,
    ImportSave,
    ImportCancel,
}
