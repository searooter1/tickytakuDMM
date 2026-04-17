use std::path::PathBuf;

use iced::widget::{image, text};
use iced::{Element, Length};

pub fn view<'a, Message: 'a>(
    thumbnail_path: Option<&'a PathBuf>,
    width: f32,
    height: f32,
    empty_text: &'a str,
) -> Element<'a, Message> {
    if let Some(path) = thumbnail_path {
        image(path.clone())
            .width(Length::Fixed(width))
            .height(Length::Fixed(height))
            .into()
    } else {
        text(empty_text).size(14).into()
    }
}