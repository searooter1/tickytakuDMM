use crate::message::{ModListAction, ModListMessage};
use crate::state::ModListState;

pub fn update(_state: &mut ModListState, message: ModListMessage) -> Option<ModListAction> {
    match message {
        ModListMessage::StartUpload => Some(ModListAction::StartUploadRequested),
        ModListMessage::Refresh => Some(ModListAction::RefreshRequested),
        ModListMessage::RemoveMod(index) => Some(ModListAction::RemoveRequested(index)),
    }
}