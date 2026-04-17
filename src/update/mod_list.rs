use crate::message::Message;
use crate::mod_manager::ModManager;
use crate::state::{AppState, ImportModState, Page};

pub(super) fn update(mod_manager: &mut ModManager, state: &mut AppState, message: Message) {
    let Page::ModList(_) = &mut state.page else {
        return;
    };

    match message {
        Message::ModListStartUpload => {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("Select a mod file")
                .add_filter("Deadlock mod package", &["vpk"])
                .pick_file()
            {
                state.page = Page::ImportMod(ImportModState::new(path));
                state.status = String::from("Fill out the mod details, then save.");
            } else {
                state.status = String::from("File selection cancelled");
            }
        }

        Message::ModListRefresh => match mod_manager.refresh() {
            Ok(()) => {
                state.status = String::from("Mod list refreshed");
            }
            Err(error) => {
                state.status = format!("Refresh failed: {error}");
            }
        },

        Message::ModListRemoveMod(index) => match mod_manager.remove_mod(index) {
            Ok(()) => {
                state.status = String::from("Mod removed");
            }
            Err(error) => {
                state.status = format!("Remove failed: {error}");
            }
        },

        _ => {}
    }
}
