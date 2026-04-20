use crate::message::Message;
use crate::mod_manager::ModManager;
use crate::state::{AppState, ModDetailsState, Page};

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
                state.page = Page::ModDetails(ModDetailsState::import(path));
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

        Message::ModListEnableMod(index) => match mod_manager.enable_mod(index) {
            Ok(path) => {
                state.status = format!("Mod enabled: copied to {}", path.display());
            }
            Err(error) => {
                state.status = format!("Enable failed: {error}");
            }
        },

        Message::ModListDisableMod(index) => match mod_manager.disable_mod(index) {
            Ok(()) => {
                state.status = String::from("Mod disabled: removed from Deadlock addons");
            }
            Err(error) => {
                state.status = format!("Disable failed: {error}");
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

        Message::ModListEditMod(index) => {
            let Some(mod_file) = mod_manager.mods.get(index) else {
                state.status = String::from("That mod is no longer in the list");
                return;
            };

            let description = mod_file.description.clone().unwrap_or_default();

            state.page = Page::ModDetails(ModDetailsState::edit(
                index,
                mod_file.file_name.clone(),
                mod_file.title.clone(),
                description,
                mod_file.thumbnail_path.clone(),
            ));
            state.status = String::from("Edit mod details, then save.");
        },

        _ => {}
    }
}
