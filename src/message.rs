#[derive(Debug, Clone)]
pub enum Message {
    ModList(ModListMessage),
    ImportMod(ImportModMessage),
}

#[derive(Debug, Clone)]
pub enum ModListMessage {
    StartUpload,
    Refresh,
    RemoveMod(usize),
}

#[derive(Debug, Clone)]
pub enum ImportModMessage {
    TitleChanged(String),
    DescriptionChanged(String),
    PickThumbnail,
    Save,
    Cancel,
}

#[derive(Debug, Clone)]
pub enum ModListAction {
    StartUploadRequested,
    RefreshRequested,
    RemoveRequested(usize),
}

#[derive(Debug, Clone)]
pub enum ImportModAction {
    PickThumbnailRequested,
    SaveRequested,
    CancelRequested,
}