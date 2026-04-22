use iced::Element;
use iced::Task;

use crate::message::Message;
use crate::mod_manager::ModManager;
use crate::state::{AppState, ModListState, Page};
use crate::update;
use crate::view::{
    gamebanana as gamebanana_view, mod_details as mod_details_view, mod_list as mod_list_view,
};

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
                page: Page::ModList(ModListState::default()),
            },
        }
    }
}

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        update::update(&mut self.mod_manager, &mut self.state, message)
    }

    pub fn view(&self) -> Element<'_, Message> {
        match &self.state.page {
            Page::ModList(state) => {
                mod_list_view::view(state, &self.mod_manager, &self.state.status)
            }

            Page::ModDetails(state) => mod_details_view::view(state, &self.state.status),

            Page::GameBanana(state) => gamebanana_view::view(state, &self.state.status),
        }
    }
}