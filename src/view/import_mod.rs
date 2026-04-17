use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Fill};

use crate::components::thumbnail;
use crate::message::ImportModMessage;
use crate::state::ImportModState;

pub fn view<'a>(
    state: &'a ImportModState,
    status: &'a str,
) -> Element<'a, ImportModMessage> {
    let thumbnail_info = state
        .thumbnail_path
        .as_ref()
        .map(|path| format!("Selected thumbnail: {}", path.display()))
        .unwrap_or_else(|| String::from("No thumbnail selected"));

    let thumbnail_preview =
        thumbnail::view(state.thumbnail_path.as_ref(), 120.0, 120.0, "No thumbnail selected");

    container(
        column![
            text("Import Mod").size(32),
            text(format!("Selected file: {}", state.mod_path.display())).size(14),
            text("Title"),
            text_input("Enter a title", &state.title).on_input(ImportModMessage::TitleChanged),
            text("Description (optional)"),
            text_input("Enter a description", &state.description)
                .on_input(ImportModMessage::DescriptionChanged),
            thumbnail_preview,
            text(thumbnail_info).size(14),
            row![
                button("Choose Thumbnail").on_press(ImportModMessage::PickThumbnail),
                button("Save Mod").on_press(ImportModMessage::Save),
                button("Cancel").on_press(ImportModMessage::Cancel),
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