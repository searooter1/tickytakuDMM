use crate::message::Message;
use crate::mod_manager::ModManager;
use crate::state::{AppState, ModListState, Page};

pub(super) fn update(mod_manager: &mut ModManager, state: &mut AppState, message: Message) {
    let Page::ImportMod(import_state) = &mut state.page else {
        return;
    };

    match message {
        Message::ImportTitleChanged(value) => {
            import_state.title = value;
        }

        Message::ImportDescriptionChanged(value) => {
            import_state.description = value;
        }

        Message::ImportPickThumbnail => {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("Select a thumbnail image")
                .add_filter("Image", &["png", "jpg", "jpeg", "webp"])
                .pick_file()
            {
                import_state.thumbnail_path = Some(path.clone());
                state.status = format!("Selected thumbnail: {}", path.display());
            } else {
                state.status = String::from("Thumbnail selection cancelled");
            }
        }

        Message::ImportSave => {
            let trimmed_title = import_state.trimmed_title();

            if trimmed_title.is_empty() {
                state.status = String::from("Title is required");
                return;
            }

            let description = import_state.trimmed_description();
            let mod_path = import_state.mod_path.clone();
            let thumbnail_path = import_state.thumbnail_path.clone();

            match mod_manager.import_file_with_metadata(
                &mod_path,
                trimmed_title,
                description,
                thumbnail_path.as_deref(),
            ) {
                Ok(saved_path) => {
                    state.status = format!("Imported to {}", saved_path.display());
                    state.page = Page::ModList(ModListState);
                }
                Err(error) => {
                    state.status = format!("Import failed: {error}");
                }
            }
        }

        Message::ImportCancel => {
            state.status = String::from("Import cancelled");
            state.page = Page::ModList(ModListState);
        }

        _ => {}
    }
}
