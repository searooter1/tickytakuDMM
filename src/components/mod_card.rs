use iced::widget::{button, column, container, row, text};
use iced::{Element, Fill};

use crate::components::thumbnail;
use crate::mod_file::ModFile;

pub fn view<'a, Message: Clone + 'a>(
    mod_file: &'a ModFile,
    on_remove: Message,
) -> Element<'a, Message> {
    let description_text = mod_file.description.as_deref().unwrap_or("No description");

    let thumbnail_widget = thumbnail::view(mod_file.thumbnail_path.as_ref(), 96.0, 96.0, "No thumbnail");

    container(
        row![
            thumbnail_widget,
            column![
                row![
                    text(&mod_file.title).size(22).width(Fill),
                    button("Remove").on_press(on_remove),
                ]
                .spacing(10),
                text(format!("File: {}", mod_file.file_name)).size(14),
                text(format!("Description: {description_text}")).size(14),
            ]
            .spacing(6)
            .width(Fill),
        ]
            .spacing(12),
    )
        .padding(12)
        .into()
}