use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Fill};

use crate::components::thumbnail;
use crate::message::Message;
use crate::state::ImportModState;

pub fn view<'a>(
    state: &'a ImportModState,
    status: &'a str,
) -> Element<'a, Message> {
    let thumbnail_preview =
        thumbnail::view(state.thumbnail_path.as_ref(), 120.0, 120.0, "No thumbnail selected");

    container(
        column![
            text("Import Mod").size(32),
            text(format!("Selected file: {}", state.mod_path.display())).size(14),
            text("Title"),
            text_input("Enter a title", &state.title).on_input(Message::ImportTitleChanged),
            text("Description (optional)"),
            text_input("Enter a description", &state.description)
                .on_input(Message::ImportDescriptionChanged),
            thumbnail_preview,
            row![
                button("Choose Thumbnail").on_press(Message::ImportPickThumbnail),
                button("Save Mod").on_press(Message::ImportSave),
                button("Cancel").on_press(Message::ImportCancel),
            ]
            .spacing(10),
            text(status),
        ]
            .spacing(16)
            .padding(20),
    )
        .width(Fill)
        .height(Fill)
        .into()
}