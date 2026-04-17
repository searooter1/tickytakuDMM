use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill};

use crate::components::mod_card;
use crate::message::Message;
use crate::mod_file::ModFile;
use crate::state::ModListState;

pub fn view<'a>(
    _state: &'a ModListState,
    mods: &'a [ModFile],
    status: &'a str,
) -> Element<'a, Message> {
    let controls = row![
        button("Upload Mod").on_press(Message::ModListStartUpload),
        button("Refresh").on_press(Message::ModListRefresh),
    ]
        .spacing(10);

    let mod_list = if mods.is_empty() {
        column![text("No mods installed")]
    } else {
        mods.iter()
            .enumerate()
            .fold(column![].spacing(15), |column, (index, mod_file)| {
                column.push(mod_card::view(
                    mod_file,
                    Message::ModListRemoveMod(index),
                ))
            })
    };

    container(
        column![
            text("Mod Manager").size(32),
            controls,
            text(status),
            scrollable(mod_list).height(Fill),
        ]
            .spacing(20)
            .padding(20),
    )
        .width(Fill)
        .height(Fill)
        .into()
}
