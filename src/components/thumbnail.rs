use std::path::PathBuf;

use iced::alignment::{self, Horizontal};
use iced::widget::{container, image, text};
use iced::{Element, Length};

use crate::ui;

pub fn view<'a, Message: 'a>(
    thumbnail_path: Option<&'a PathBuf>,
    width: f32,
    height: f32,
    empty_text: &'a str,
) -> Element<'a, Message> {
    let inner: Element<'a, Message> = if let Some(path) = thumbnail_path {
        image(path.clone())
            .width(Length::Fixed(width))
            .height(Length::Fixed(height))
            .into()
    } else {
        container(
            text(empty_text)
                .size(12)
                .style(|theme| text::secondary(theme)),
        )
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .align_x(Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .into()
    };

    container(inner)
        .style(ui::media_frame)
        .clip(true)
        .into()
}
