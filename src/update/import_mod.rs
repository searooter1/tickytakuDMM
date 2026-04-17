use crate::message::{ImportModAction, ImportModMessage};
use crate::state::ImportModState;

pub fn update(state: &mut ImportModState, message: ImportModMessage) -> Option<ImportModAction> {
    match message {
        ImportModMessage::TitleChanged(value) => {
            state.title = value;
            None
        }

        ImportModMessage::DescriptionChanged(value) => {
            state.description = value;
            None
        }

        ImportModMessage::PickThumbnail => Some(ImportModAction::PickThumbnailRequested),

        ImportModMessage::Save => Some(ImportModAction::SaveRequested),

        ImportModMessage::Cancel => Some(ImportModAction::CancelRequested),
    }
}