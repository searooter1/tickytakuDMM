use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Fill};

use crate::components::thumbnail;
use crate::message::Message;
use crate::state::{ModDetailsMode, ModDetailsState};

pub fn view<'a>(state: &'a ModDetailsState, status: &'a str) -> Element<'a, Message> {
    let (heading, context_line) = match &state.mode {
        ModDetailsMode::Import { source_path } => (
            "Import Mod",
            format!("Selected file: {}", source_path.display()),
        ),
        ModDetailsMode::Edit { file_name, .. } => ("Edit Mod", format!("Mod file: {file_name}")),
    };

    let thumbnail_preview =
        thumbnail::view(state.thumbnail_path.as_ref(), 120.0, 120.0, "No thumbnail selected");

    container(
        column![
            text(heading).size(32),
            text(context_line).size(14),
            text("Title"),
            text_input("Enter a title", &state.title).on_input(Message::ModDetailsTitleChanged),
            text("Description (optional)"),
            text_input("Enter a description", &state.description)
                .on_input(Message::ModDetailsDescriptionChanged),
            thumbnail_preview,
            row![
                button("Choose Thumbnail").on_press(Message::ModDetailsPickThumbnail),
                button("Clear Thumbnail").on_press(Message::ModDetailsClearThumbnail),
                button("Save").on_press(Message::ModDetailsSave),
                button("Cancel").on_press(Message::ModDetailsCancel),
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
