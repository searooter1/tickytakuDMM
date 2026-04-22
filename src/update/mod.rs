mod gamebanana;
mod mod_details;
mod mod_list;

use iced::Task;

use crate::message::Message;
use crate::mod_manager::ModManager;
use crate::state::{AppState, GameBananaState, Page};

pub fn update(mod_manager: &mut ModManager, state: &mut AppState, message: Message) -> Task<Message> {
    if let Message::ModListOpenGameBanana = message {
        state.page = Page::GameBanana(GameBananaState::new_browse());
        return gamebanana::initial_list_task();
    }

    if is_gamebanana_message(&message) {
        return gamebanana::update(mod_manager, state, message);
    }

    match message {
        Message::ModListStartUpload
        | Message::ModListRefresh
        | Message::ModListEnableMod(_)
        | Message::ModListDisableMod(_)
        | Message::ModListRemoveMod(_)
        | Message::ModListEditMod(_)
        | Message::ModListMoveModUp(_)
        | Message::ModListMoveModDown(_) => {
            mod_list::update(mod_manager, state, message);
            Task::none()
        }

        Message::ModDetailsTitleChanged(_)
        | Message::ModDetailsDescriptionChanged(_)
        | Message::ModDetailsPickThumbnail
        | Message::ModDetailsClearThumbnail
        | Message::ModDetailsSave
        | Message::ModDetailsCancel => {
            mod_details::update(mod_manager, state, message);
            Task::none()
        }

        _ => Task::none(),
    }
}

fn is_gamebanana_message(message: &Message) -> bool {
    matches!(
        message,
        Message::GameBananaBack
            | Message::GameBananaBrowseCategorySelected(_)
            | Message::GameBananaSearchInput(_)
            | Message::GameBananaSearchSubmit
            | Message::GameBananaBrowseMode
            | Message::GameBananaPagePrev
            | Message::GameBananaPageNext
            | Message::GameBananaSelectMod(_)
            | Message::GameBananaThumbCarousel { .. }
            | Message::GameBananaOpenModUrl(_)
            | Message::GameBananaDownloadFile(_)
            | Message::GameBananaListLoaded { .. }
            | Message::GameBananaThumbnailsReady { .. }
            | Message::GameBananaFilesLoaded { .. }
            | Message::GameBananaImportDone(_)
    )
}
