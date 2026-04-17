use iced::Element;

use crate::message::Message;
use crate::mod_manager::ModManager;
use crate::state::{AppState, ModListState, Page};
use crate::update;
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
        update::update(&mut self.mod_manager, &mut self.state, message);
    }

    pub fn view(&self) -> Element<'_, Message> {
        match &self.state.page {
            Page::ModList(state) => {
                mod_list_view::view(state, &self.mod_manager.mods, &self.state.status)
            }

            Page::ImportMod(state) => import_mod_view::view(state, &self.state.status),
        }
    }
}