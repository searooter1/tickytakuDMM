mod import_mod;
mod mod_list;

use crate::message::Message;
use crate::mod_manager::ModManager;
use crate::state::AppState;

pub fn update(mod_manager: &mut ModManager, state: &mut AppState, message: Message) {
    match message {
        Message::ModListStartUpload
        | Message::ModListRefresh
        | Message::ModListRemoveMod(_) => mod_list::update(mod_manager, state, message),

        Message::ImportTitleChanged(_)
        | Message::ImportDescriptionChanged(_)
        | Message::ImportPickThumbnail
        | Message::ImportSave
        | Message::ImportCancel => import_mod::update(mod_manager, state, message),
    }
}
