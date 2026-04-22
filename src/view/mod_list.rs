use iced::alignment::{self, Horizontal};
use iced::font::{Font, Weight};
use iced::widget::button;
use iced::widget::{column, container, row, rule, scrollable, text};
use iced::{padding, Element, Fill};

use crate::components::mod_card;
use crate::message::Message;
use crate::mod_manager::ModManager;
use crate::state::ModListState;
use crate::ui;

pub fn view<'a>(
    _list_state: &'a ModListState,
    mod_manager: &'a ModManager,
    status: &'a str,
) -> Element<'a, Message> {
    let mods = &mod_manager.mods;

    let controls = row![
        button(text("Upload mod").size(15))
            .padding([11, 20])
            .on_press(Message::ModListStartUpload)
            .style(|theme, status| button::secondary(theme, status)),
        button(text("GameBanana").size(15))
            .padding([11, 18])
            .on_press(Message::ModListOpenGameBanana)
            .style(|theme, status| button::secondary(theme, status)),
        button(text("Refresh").size(15))
            .padding([11, 18])
            .on_press(Message::ModListRefresh)
            .style(|theme, status| button::secondary(theme, status)),
    ]
    .spacing(12)
    .align_y(alignment::Vertical::Center);

    let heading = text("TICKYTAKU'S DMM")
        .size(32)
        .font(Font {
            weight: Weight::Semibold,
            ..Font::DEFAULT
        });

    let subtitle = text("Deadlock mods live here — upload or grab some from GameBanana, then enable what you want in-game. Lower pak numbers load first. Use Up / Down to change load order.")
        .size(14)
        .style(|theme| text::secondary(theme))
        .wrapping(text::Wrapping::Word);

    let title_block = column![heading, subtitle]
        .spacing(8)
        .align_x(Horizontal::Left)
        .width(Fill);

    let nav_inner = container(
        row![title_block.width(Fill), controls]
            .spacing(28)
            .align_y(alignment::Vertical::Center),
    )
    .width(Fill);

    let nav = container(nav_inner)
        .width(Fill)
        .padding([24, 40])
        .style(ui::nav_bar);

    let divider = rule::horizontal(1).style(rule::weak);

    let mod_list: Element<'a, Message> = if mods.is_empty() {
        container(
            column![
                text("Nothing installed yet")
                    .size(20)
                    .font(Font {
                        weight: Weight::Semibold,
                        ..Font::DEFAULT
                    })
                    .style(|theme| text::primary(theme)),
                text("Upload a .vpk to add it to your library. Mods are saved as pak01_dir.vpk … pak99_dir.vpk (lower = higher priority). Enable copies into Deadlock’s addons and fixes citadel/gameinfo.gi SearchPaths when needed (game updates may reset that file).")
                    .size(14)
                    .style(|theme| text::secondary(theme))
                    .wrapping(text::Wrapping::Word),
            ]
            .spacing(10)
            .align_x(Horizontal::Left)
            .width(Fill),
        )
        .padding([48, 32])
        .width(Fill)
        .align_x(Horizontal::Left)
        .style(ui::surface_card)
        .into()
    } else {
        let n = mods.len();
        let rows = mods.iter().enumerate().fold(column![].spacing(12), |col, (index, mod_file)| {
            let enabled = mod_manager.is_mod_enabled(index);
            let order_up = (index > 0).then_some(Message::ModListMoveModUp(index));
            let order_down = (index + 1 < n).then_some(Message::ModListMoveModDown(index));
            col.push(mod_card::view(
                mod_file,
                enabled,
                if enabled {
                    Message::ModListDisableMod(index)
                } else {
                    Message::ModListEnableMod(index)
                },
                Message::ModListEditMod(index),
                Message::ModListRemoveMod(index),
                order_up,
                order_down,
            ))
        });
        scrollable(rows).height(Fill).into()
    };

    let status_line = text(status)
        .size(13)
        .style(|theme| text::secondary(theme))
        .wrapping(text::Wrapping::Word);

    let status_bar = container(status_line)
        .padding([12, 16])
        .width(Fill)
        .style(ui::status_footer);

    let body = container(
        column![mod_list, status_bar]
            .spacing(18)
            .max_width(ui::CONTENT_MAX_WIDTH),
    )
    .width(Fill)
    .height(Fill)
    .align_x(alignment::Horizontal::Center)
    .padding(padding::top(22).bottom(32).left(40).right(40));

    container(column![nav, divider, body].spacing(0))
        .width(Fill)
        .height(Fill)
        .style(ui::page_backdrop)
        .into()
}
