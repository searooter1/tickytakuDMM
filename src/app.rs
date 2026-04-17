use iced::Element;

use crate::message::{
    ImportModAction, ImportModMessage, Message, ModListAction, ModListMessage,
};
use crate::mod_manager::ModManager;
use crate::state::{AppState, ImportModState, ModListState, Page};
use crate::update::{import_mod as import_mod_update, mod_list as mod_list_update};
use crate::view::{import_mod as import_mod_view, mod_list as mod_list_view};

#[derive(Debug)]
pub struct App {
    mod_manager: ModManager,
    state: AppState,
}

impl Default for App {
    fn default() -> Self {
        let mod_manager = ModManager::new();

        let status = match ModManager::mods_dir() {
            Ok(path) => format!("Mods folder: {}", path.display()),
            Err(error) => error,
        };

        Self {
            mod_manager,
            state: AppState {
                status,
                page: Page::ModList(ModListState),
            },
        }
    }
}

impl App {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ModList(message) => self.update_mod_list(message),
            Message::ImportMod(message) => self.update_import_mod(message),
        }
    }

    fn update_mod_list(&mut self, message: ModListMessage) {
        if let Page::ModList(state) = &mut self.state.page {
            let action = mod_list_update::update(state, message);

            match action {
                None => {}

                Some(ModListAction::StartUploadRequested) => {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_title("Select a mod file")
                        .add_filter("Deadlock mod package", &["vpk"])
                        .pick_file()
                    {
                        self.state.page = Page::ImportMod(ImportModState::new(path));
                        self.state.status =
                            String::from("Fill out the mod details, then save.");
                    } else {
                        self.state.status = String::from("File selection cancelled");
                    }
                }

                Some(ModListAction::RefreshRequested) => match self.mod_manager.refresh() {
                    Ok(()) => {
                        self.state.status = String::from("Mod list refreshed");
                    }
                    Err(error) => {
                        self.state.status = format!("Refresh failed: {error}");
                    }
                },

                Some(ModListAction::RemoveRequested(index)) => {
                    match self.mod_manager.remove_mod(index) {
                        Ok(()) => {
                            self.state.status = String::from("Mod removed");
                        }
                        Err(error) => {
                            self.state.status = format!("Remove failed: {error}");
                        }
                    }
                }
            }
        }
    }

    fn update_import_mod(&mut self, message: ImportModMessage) {
        let mut next_page: Option<Page> = None;

        if let Page::ImportMod(state) = &mut self.state.page {
            let action = import_mod_update::update(state, message);

            match action {
                None => {}

                Some(ImportModAction::PickThumbnailRequested) => {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_title("Select a thumbnail image")
                        .add_filter("Image", &["png", "jpg", "jpeg", "webp"])
                        .pick_file()
                    {
                        state.thumbnail_path = Some(path.clone());
                        self.state.status = format!("Selected thumbnail: {}", path.display());
                    } else {
                        self.state.status = String::from("Thumbnail selection cancelled");
                    }
                }

                Some(ImportModAction::SaveRequested) => {
                    let trimmed_title = state.trimmed_title();

                    if trimmed_title.is_empty() {
                        self.state.status = String::from("Title is required");
                        return;
                    }

                    let description = state.trimmed_description();

                    match self.mod_manager.import_file_with_metadata(
                        &state.mod_path,
                        trimmed_title,
                        description,
                        state.thumbnail_path.as_deref(),
                    ) {
                        Ok(saved_path) => {
                            self.state.status = format!("Imported to {}", saved_path.display());
                            next_page = Some(Page::ModList(ModListState));
                        }
                        Err(error) => {
                            self.state.status = format!("Import failed: {error}");
                        }
                    }
                }

                Some(ImportModAction::CancelRequested) => {
                    self.state.status = String::from("Import cancelled");
                    next_page = Some(Page::ModList(ModListState));
                }
            }
        }

        if let Some(page) = next_page {
            self.state.page = page;
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        match &self.state.page {
            Page::ModList(state) => mod_list_view::view(
                state,
                &self.mod_manager.mods,
                &self.state.status,
            )
                .map(Message::ModList),

            Page::ImportMod(state) => import_mod_view::view(state, &self.state.status)
                .map(Message::ImportMod),
        }
    }
}