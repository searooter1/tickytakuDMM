use std::path::PathBuf;

use iced::widget::{button, column, container, image, row, text, text_input};
use iced::{Element, Fill, Length};

#[derive(Debug, Clone)]
pub struct State {
    pub mod_path: PathBuf,
    pub title: String,
    pub description: String,
    pub thumbnail_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum Message {
    TitleChanged(String),
    DescriptionChanged(String),
    PickThumbnail,
    Save,
    Cancel,
}

impl State {
    pub fn new(mod_path: PathBuf) -> Self {
        let title = mod_path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("New Mod")
            .to_string();

        Self {
            mod_path,
            title,
            description: String::new(),
            thumbnail_path: None,
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::TitleChanged(value) => {
                self.title = value;
            }
            Message::DescriptionChanged(value) => {
                self.description = value;
            }
            Message::PickThumbnail | Message::Save | Message::Cancel => {}
        }
    }

    pub fn trimmed_title(&self) -> String {
        self.title.trim().to_string()
    }

    pub fn trimmed_description(&self) -> Option<String> {
        let trimmed = self.description.trim();

        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }

    pub fn view<'a>(&'a self, status: &'a str) -> Element<'a, Message> {
        let thumbnail_info = self
            .thumbnail_path
            .as_ref()
            .map(|path| format!("Selected thumbnail: {}", path.display()))
            .unwrap_or_else(|| String::from("No thumbnail selected"));

        let thumbnail_preview: Element<'a, Message> = if let Some(path) = &self.thumbnail_path {
            image(path.clone())
                .width(Length::Fixed(120.0))
                .height(Length::Fixed(120.0))
                .into()
        } else {
            text("No thumbnail selected").size(14).into()
        };

        container(
            column![
                text("Import Mod").size(32),
                text(format!("Selected file: {}", self.mod_path.display())).size(14),
                text("Title"),
                text_input("Enter a title", &self.title).on_input(Message::TitleChanged),
                text("Description (optional)"),
                text_input("Enter a description", &self.description)
                    .on_input(Message::DescriptionChanged),
                thumbnail_preview,
                text(thumbnail_info).size(14),
                row![
                    button("Choose Thumbnail").on_press(Message::PickThumbnail),
                    button("Save Mod").on_press(Message::Save),
                    button("Cancel").on_press(Message::Cancel),
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
}