use iced::widget::{button, column, container, image, row, scrollable, text};
use iced::{Element, Fill, Length};

use crate::mod_file::ModFile;

#[derive(Debug, Default, Clone)]
pub struct State;

#[derive(Debug, Clone)]
pub enum Message {
    StartUpload,
    Refresh,
    RemoveMod(usize),
}

impl State {
    pub fn update(&mut self, _message: Message) {
        // This page currently has no local state.
        // The method still exists because in Elm architecture
        // each page owns its own update function.
    }

    pub fn view<'a>(&'a self, mods: &'a [ModFile], status: &'a str) -> Element<'a, Message> {
        let controls = row![
            button("Upload Mod").on_press(Message::StartUpload),
            button("Refresh").on_press(Message::Refresh),
        ]
            .spacing(10);

        let mod_list = if mods.is_empty() {
            column![text("No mods installed")]
        } else {
            mods.iter()
                .enumerate()
                .fold(column![].spacing(15), |column, (index, mod_file)| {
                    let description_text =
                        mod_file.description.as_deref().unwrap_or("No description");

                    let thumbnail_widget: Element<'a, Message> =
                        if let Some(path) = &mod_file.thumbnail_path {
                            image(path.clone())
                                .width(Length::Fixed(96.0))
                                .height(Length::Fixed(96.0))
                                .into()
                        } else {
                            text("No thumbnail").size(14).into()
                        };

                    column.push(
                        container(
                            row![
                                thumbnail_widget,
                                column![
                                    row![
                                        text(&mod_file.title).size(22).width(Fill),
                                        button("Remove").on_press(Message::RemoveMod(index)),
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
                            .padding(12),
                    )
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
}