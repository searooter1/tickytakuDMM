#[derive(Debug, Clone)]
pub enum Message {
    // Main screen actions
    StartUploadMod,
    RemoveMod(usize),
    RefreshMods,

    // Import form actions
    ImportTitleChanged(String),
    ImportDescriptionChanged(String),
    PickThumbnail,
    SaveImport,
    CancelImport,
}