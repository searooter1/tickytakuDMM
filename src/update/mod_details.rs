use crate::message::Message;
use crate::mod_manager::ModManager;
use crate::state::{AppState, ModDetailsMode, ModListState, Page};

pub(super) fn update(mod_manager: &mut ModManager, state: &mut AppState, message: Message) {
    let Page::ModDetails(details) = &mut state.page else {
        return;
    };

    match message {
        Message::ModDetailsTitleChanged(value) => {
            details.title = value;
        }

        Message::ModDetailsDescriptionChanged(value) => {
            details.description = value;
        }

        Message::ModDetailsPickThumbnail => {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("Select a thumbnail image")
                .add_filter("Image", &["png", "jpg", "jpeg", "webp"])
                .pick_file()
            {
                details.thumbnail_path = Some(path.clone());
                state.status = format!("Selected thumbnail: {}", path.display());
            } else {
                state.status = String::from("Thumbnail selection cancelled");
            }
        }

        Message::ModDetailsClearThumbnail => {
            details.thumbnail_path = None;
            state.status = String::from("Thumbnail cleared (not saved until you click Save)");
        }

        Message::ModDetailsSave => {
            let trimmed_title = details.trimmed_title();

            if trimmed_title.is_empty() {
                state.status = String::from("Title is required");
                return;
            }

            let description = details.trimmed_description();
            let thumbnail_path = details.thumbnail_path.clone();

            match &details.mode {
                ModDetailsMode::Import { source_path } => {
                    let mod_path = source_path.clone();

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

                ModDetailsMode::Edit {
                    mod_index,
                    original_thumbnail_path,
                    ..
                } => match mod_manager.update_mod_entry(
                    *mod_index,
                    trimmed_title,
                    description,
                    thumbnail_path,
                    original_thumbnail_path.clone(),
                ) {
                    Ok(()) => {
                        state.status = String::from("Mod updated");
                        state.page = Page::ModList(ModListState);
                    }
                    Err(error) => {
                        state.status = format!("Save failed: {error}");
                    }
                },
            }
        }

        Message::ModDetailsCancel => {
            state.status = match &details.mode {
                ModDetailsMode::Import { .. } => String::from("Import cancelled"),
                ModDetailsMode::Edit { .. } => String::from("Edit cancelled"),
            };
            state.page = Page::ModList(ModListState);
        }

        _ => {}
    }
}
