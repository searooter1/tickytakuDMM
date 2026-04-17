use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill};

use crate::components::mod_card;
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
        // We still keep the method because each page in this
        // architecture owns its own update function.
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
                    column.push(mod_card::view(mod_file, Message::RemoveMod(index)))
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