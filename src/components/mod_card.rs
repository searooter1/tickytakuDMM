use iced::alignment;
use iced::font::{Font, Weight};
use iced::widget::button;
use iced::widget::{column, container, row, space, text};
use iced::{Element, Fill, Length};

use crate::components::thumbnail;
use crate::mod_file::ModFile;
use crate::ui;

pub fn view<'a, Message: Clone + 'a>(
    mod_file: &'a ModFile,
    enabled: bool,
    on_toggle: Message,
    on_edit: Message,
    on_remove: Message,
    order_up: Option<Message>,
    order_down: Option<Message>,
) -> Element<'a, Message> {
    let description_text = mod_file.description.as_deref().unwrap_or("No description");

    let thumbnail_widget = thumbnail::view(mod_file.thumbnail_path.as_ref(), 96.0, 96.0, "No art");

    let title = text(&mod_file.title)
        .size(19)
        .font(Font {
            weight: Weight::Semibold,
            ..Font::DEFAULT
        })
        .width(Fill);

    let meta = column![
        text(format!("File: {}", mod_file.file_name))
            .size(12)
            .style(|theme| text::secondary(theme)),
        text(description_text)
            .size(13)
            .style(|theme| text::secondary(theme))
            .wrapping(text::Wrapping::Word),
    ]
    .spacing(6)
    .width(Fill);

    let toggle_button = if enabled {
        button(text("Disable").size(14))
            .padding([9, 16])
            .on_press(on_toggle)
            .style(|theme, status| button::secondary(theme, status))
    } else {
        button(text("Enable").size(14))
            .padding([9, 16])
            .on_press(on_toggle)
            .style(|theme, status| button::secondary(theme, status))
    };

    let up_btn: Element<'a, Message> = if let Some(m) = order_up {
        button(text("Up").size(12))
            .padding([7, 10])
            .on_press(m)
            .style(|theme, status| button::secondary(theme, status))
            .into()
    } else {
        space()
            .width(Length::Fixed(52.0))
            .height(Length::Fixed(32.0))
            .into()
    };

    let down_btn: Element<'a, Message> = if let Some(m) = order_down {
        button(text("Down").size(12))
            .padding([7, 10])
            .on_press(m)
            .style(|theme, status| button::secondary(theme, status))
            .into()
    } else {
        space()
            .width(Length::Fixed(58.0))
            .height(Length::Fixed(32.0))
            .into()
    };

    let order_buttons = column![up_btn, down_btn]
        .spacing(6)
        .align_x(alignment::Horizontal::Center);

    let actions = row![
        order_buttons,
        toggle_button,
        button(text("Edit").size(14))
            .padding([9, 16])
            .on_press(on_edit)
            .style(|theme, status| button::secondary(theme, status)),
        button(text("Remove").size(14))
            .padding([9, 14])
            .on_press(on_remove)
            .style(|theme, status| button::secondary(theme, status)),
    ]
    .spacing(8)
    .align_y(alignment::Vertical::Center);

    let body = column![
        row![title, actions]
            .spacing(20)
            .align_y(alignment::Vertical::Center),
        meta,
    ]
    .spacing(12)
    .width(Fill);

    container(
        row![thumbnail_widget, body]
            .spacing(18)
            .align_y(alignment::Vertical::Center),
    )
    .padding([20, 22])
    .width(Fill)
    .style(ui::surface_card)
    .into()
}
