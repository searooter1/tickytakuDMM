mod mod_details;
mod mod_list;

use crate::message::Message;
use crate::mod_manager::ModManager;
use crate::state::AppState;

pub fn update(mod_manager: &mut ModManager, state: &mut AppState, message: Message) {
    match message {
        Message::ModListStartUpload
        | Message::ModListRefresh
        | Message::ModListRemoveMod(_)
        | Message::ModListEditMod(_) => mod_list::update(mod_manager, state, message),

        Message::ModDetailsTitleChanged(_)
        | Message::ModDetailsDescriptionChanged(_)
        | Message::ModDetailsPickThumbnail
        | Message::ModDetailsClearThumbnail
        | Message::ModDetailsSave
        | Message::ModDetailsCancel => mod_details::update(mod_manager, state, message),
    }
}
