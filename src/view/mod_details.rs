use iced::alignment;
use iced::font::{Font, Weight};
use iced::widget::button;
use iced::widget::{column, container, row, rule, text, text_input};
use iced::{padding, Element, Fill};

use crate::components::thumbnail;
use crate::message::Message;
use crate::state::{ModDetailsMode, ModDetailsState};
use crate::ui;

pub fn view<'a>(state: &'a ModDetailsState, status: &'a str) -> Element<'a, Message> {
    let (heading, context_line) = match &state.mode {
        ModDetailsMode::Import { source_path } => (
            "Import mod",
            format!("Source: {}", source_path.display()),
        ),
        ModDetailsMode::Edit { file_name, .. } => ("Edit mod", format!("File: {file_name}")),
    };

    let title_label = text("Title")
        .size(12)
        .style(|theme| text::secondary(theme));

    let desc_label = text("Description (optional)")
        .size(12)
        .style(|theme| text::secondary(theme));

    let title_field = text_input("Enter a title", &state.title)
        .on_input(Message::ModDetailsTitleChanged)
        .padding([14, 16])
        .size(16);

    let description_field = text_input("Enter a description", &state.description)
        .on_input(Message::ModDetailsDescriptionChanged)
        .padding([14, 16])
        .size(16);

    let thumbnail_preview = thumbnail::view(
        state.thumbnail_path.as_ref(),
        128.0,
        128.0,
        "No thumbnail",
    );

    let heading_text = text(heading)
        .size(30)
        .font(Font {
            weight: Weight::Semibold,
            ..Font::DEFAULT
        });

    let context = text(context_line)
        .size(14)
        .style(|theme| text::secondary(theme))
        .wrapping(text::Wrapping::Word);

    let actions = row![
        button(text("Choose thumbnail").size(15))
            .padding([11, 18])
            .on_press(Message::ModDetailsPickThumbnail)
            .style(|theme, status| button::secondary(theme, status)),
        button(text("Clear").size(15))
            .padding([11, 16])
            .on_press(Message::ModDetailsClearThumbnail)
            .style(|theme, status| button::secondary(theme, status)),
        row![].width(Fill),
        button(text("Cancel").size(15))
            .padding([11, 16])
            .on_press(Message::ModDetailsCancel)
            .style(|theme, status| button::secondary(theme, status)),
        button(text("Save").size(15))
            .padding([11, 22])
            .on_press(Message::ModDetailsSave)
            .style(|theme, status| button::secondary(theme, status)),
    ]
    .spacing(10)
    .align_y(alignment::Vertical::Center);

    let status_chip = container(
        text(status)
            .size(13)
            .style(|theme| text::secondary(theme))
            .wrapping(text::Wrapping::Word),
    )
    .padding([12, 16])
    .width(Fill)
    .style(ui::status_footer);

    let form = column![
        heading_text,
        context,
        rule::horizontal(1).style(rule::weak),
        title_label,
        title_field,
        desc_label,
        description_field,
        text("Thumbnail")
            .size(12)
            .style(|theme| text::secondary(theme)),
        thumbnail_preview,
        actions,
        status_chip,
    ]
    .spacing(16)
    .max_width(560);

    let panel = container(form)
        .padding([32, 36])
        .width(Fill)
        .align_x(alignment::Horizontal::Center)
        .style(ui::surface_card);

    let body = container(column![panel].max_width(ui::CONTENT_MAX_WIDTH))
        .width(Fill)
        .height(Fill)
        .align_x(alignment::Horizontal::Center)
        .padding(padding::top(32).bottom(40).left(40).right(40));

    container(body)
        .width(Fill)
        .height(Fill)
        .style(ui::page_backdrop)
        .into()
}
