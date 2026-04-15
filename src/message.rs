#[derive(Debug, Clone)]
pub enum Message {
    UploadMod,
    RemoveMod(usize),
    RefreshMods,
}