use iced::Element;

use crate::mod_manager::ModManager;
use crate::pages::{import_mod, mod_list};

#[derive(Debug)]
pub struct App {
    mod_manager: ModManager,
    status: String,
    page: Page,
}

#[derive(Debug)]
enum Page {
    ModList(mod_list::State),
    ImportMod(import_mod::State),
}

#[derive(Debug, Clone)]
pub enum Message {
    ModList(mod_list::Message),
    ImportMod(import_mod::Message),
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
            status,
            page: Page::ModList(mod_list::State),
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

    fn update_mod_list(&mut self, message: mod_list::Message) {
        if let Page::ModList(state) = &mut self.page {
            state.update(message.clone());
        }

        match message {
            mod_list::Message::StartUpload => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("Select a mod file")
                    .add_filter("Deadlock mod package", &["vpk"])
                    .pick_file()
                {
                    self.page = Page::ImportMod(import_mod::State::new(path));
                    self.status = String::from("Fill out the mod details, then save.");
                } else {
                    self.status = String::from("File selection cancelled");
                }
            }

            mod_list::Message::Refresh => match self.mod_manager.refresh() {
                Ok(()) => {
                    self.status = String::from("Mod list refreshed");
                }
                Err(error) => {
                    self.status = format!("Refresh failed: {error}");
                }
            },

            mod_list::Message::RemoveMod(index) => match self.mod_manager.remove_mod(index) {
                Ok(()) => {
                    self.status = String::from("Mod removed");
                }
                Err(error) => {
                    self.status = format!("Remove failed: {error}");
                }
            },
        }
    }

    fn update_import_mod(&mut self, message: import_mod::Message) {
        let mut next_page: Option<Page> = None;

        if let Page::ImportMod(state) = &mut self.page {
            match &message {
                import_mod::Message::TitleChanged(_)
                | import_mod::Message::DescriptionChanged(_) => {
                    state.update(message.clone());
                }

                import_mod::Message::PickThumbnail => {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_title("Select a thumbnail image")
                        .add_filter("Image", &["png", "jpg", "jpeg", "webp"])
                        .pick_file()
                    {
                        state.thumbnail_path = Some(path.clone());
                        self.status = format!("Selected thumbnail: {}", path.display());
                    } else {
                        self.status = String::from("Thumbnail selection cancelled");
                    }
                }

                import_mod::Message::Save => {
                    let trimmed_title = state.trimmed_title();

                    if trimmed_title.is_empty() {
                        self.status = String::from("Title is required");
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
                            self.status = format!("Imported to {}", saved_path.display());
                            next_page = Some(Page::ModList(mod_list::State));
                        }
                        Err(error) => {
                            self.status = format!("Import failed: {error}");
                        }
                    }
                }

                import_mod::Message::Cancel => {
                    self.status = String::from("Import cancelled");
                    next_page = Some(Page::ModList(mod_list::State));
                }
            }
        }

        if let Some(page) = next_page {
            self.page = page;
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        match &self.page {
            Page::ModList(state) => state
                .view(&self.mod_manager.mods, &self.status)
                .map(Message::ModList),
            Page::ImportMod(state) => state.view(&self.status).map(Message::ImportMod),
        }
    }
}