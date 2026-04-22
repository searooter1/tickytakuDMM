use iced::alignment::{self, Horizontal};
use iced::font::{Font, Weight};
use iced::widget::button;
use iced::widget::image::Handle;
use iced::widget::{column, container, image, row, rule, scrollable, text, text_input, Space};
use iced::{padding, ContentFit, Element, Fill, Length};

use crate::gamebanana::{self, FileEntry, ModSummary};
use crate::message::Message;
use crate::state::{GameBananaBrowseCategory, GameBananaListSource, GameBananaState};
use crate::ui;

fn max_page(total: u64, per: u32) -> u64 {
    let per = per as u64;
    if total == 0 {
        1
    } else {
        (total + per - 1) / per
    }
}

/// Preview size in the mod list (carousel).
const LIST_CAROUSEL: f32 = 132.0;
/// Preview size in the detail column when a mod is selected.
const DETAIL_CAROUSEL: f32 = 208.0;

fn carousel_nav_button<'a>(label: &'a str, mod_id: u64, next: bool) -> Element<'a, Message> {
    button(text(label).size(18))
        .padding([4, 10])
        .on_press(Message::GameBananaThumbCarousel { mod_id, next })
        .style(|theme, status| button::secondary(theme, status))
        .into()
}

fn mod_carousel<'a>(
    mod_id: u64,
    handles: &'a [Handle],
    index: usize,
    tile: f32,
) -> Element<'a, Message> {
    let placeholder: Element<'a, Message> = container(
        text("…")
            .size(18)
            .style(|theme| text::secondary(theme)),
    )
    .width(Length::Fixed(tile))
    .height(Length::Fixed(tile))
    .align_x(Horizontal::Center)
    .align_y(alignment::Vertical::Center)
    .into();

    let inner: Element<'a, Message> = if handles.is_empty() {
        placeholder
    } else {
        let len = handles.len();
        let idx = index % len;
        let current = &handles[idx];
        let img = image(current.clone())
            .width(Length::Fixed(tile))
            .height(Length::Fixed(tile))
            .content_fit(ContentFit::Cover);

        let counter: Element<'a, Message> = if len > 1 {
            text(format!("{} / {}", idx + 1, len))
                .size(11)
                .style(|theme| text::secondary(theme))
                .into()
        } else {
            Space::new().height(0).into()
        };

        let prev: Element<'a, Message> = if len > 1 {
            carousel_nav_button("‹", mod_id, false)
        } else {
            Space::new().width(Length::Fixed(36.0)).into()
        };

        let next: Element<'a, Message> = if len > 1 {
            carousel_nav_button("›", mod_id, true)
        } else {
            Space::new().width(Length::Fixed(36.0)).into()
        };

        column![
            row![prev, container(img).clip(true), next]
                .spacing(6)
                .align_y(alignment::Vertical::Center),
            counter,
        ]
        .spacing(4)
        .align_x(Horizontal::Center)
        .into()
    };

    container(inner)
        .style(ui::media_frame)
        .clip(true)
        .into()
}

fn mod_row<'a>(m: &'a ModSummary, gb: &'a GameBananaState, selected: bool) -> Element<'a, Message> {
    let handles: &[Handle] = gb
        .thumbnails
        .get(&m.id)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let idx = *gb.thumb_carousel_index.get(&m.id).unwrap_or(&0);

    let text_block = column![
        text(&m.name).size(14).wrapping(text::Wrapping::Word),
        if m.has_files {
            text("Has downloads").size(11)
        } else {
            text("No files").size(11)
        }
        .style(|theme| text::secondary(theme)),
    ]
    .spacing(4)
    .align_x(Horizontal::Left)
    .width(Fill);

    let mut select_btn = button(text_block)
        .padding([10, 12])
        .width(Fill)
        .on_press(Message::GameBananaSelectMod(m.id));

    if selected {
        select_btn = select_btn.style(|theme, status| button::primary(theme, status));
    } else {
        select_btn = select_btn.style(|theme, status| button::secondary(theme, status));
    }

    row![
        mod_carousel(m.id, handles, idx, LIST_CAROUSEL),
        select_btn,
    ]
    .spacing(14)
    .align_y(alignment::Vertical::Center)
    .into()
}

fn detail_preview_carousel<'a>(gb: &'a GameBananaState) -> Element<'a, Message> {
    let Some(mod_id) = gb.selected_mod_id else {
        return Space::new().height(0).into();
    };
    let handles: &[Handle] = gb
        .thumbnails
        .get(&mod_id)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let idx = *gb.thumb_carousel_index.get(&mod_id).unwrap_or(&0);

    if handles.is_empty() && gb.mods.iter().any(|m| m.id == mod_id && !m.preview_image_urls.is_empty())
    {
        return text("Loading previews…")
            .size(12)
            .style(|theme| text::secondary(theme))
            .into();
    }

    if handles.is_empty() {
        return Space::new().height(0).into();
    }

    column![
        text("Screenshots")
            .size(13)
            .font(Font {
                weight: Weight::Semibold,
                ..Font::DEFAULT
            }),
        mod_carousel(mod_id, handles, idx, DETAIL_CAROUSEL),
    ]
    .spacing(8)
    .into()
}

fn file_row<'a>(f: &'a FileEntry, busy: bool) -> Element<'a, Message> {
    let size_mb = f.size_bytes as f64 / 1_048_576.0;
    let desc = f
        .description
        .as_deref()
        .unwrap_or("")
        .chars()
        .take(80)
        .collect::<String>();

    let label = format!(
        "{} — {:.1} MiB{}",
        f.file_name,
        size_mb,
        if desc.is_empty() {
            String::new()
        } else {
            format!(" — {desc}")
        }
    );

    let mut dl = button(text("Download & import").size(13))
        .padding([8, 14])
        .on_press_maybe((!busy).then(|| Message::GameBananaDownloadFile(f.clone())));

    dl = if busy {
        dl.style(|theme, status| button::secondary(theme, status))
    } else {
        dl.style(|theme, status| button::primary(theme, status))
    };

    row![
        text(label)
            .size(13)
            .wrapping(text::Wrapping::Word)
            .width(Fill),
        dl,
    ]
    .spacing(12)
    .align_y(alignment::Vertical::Center)
    .into()
}

pub fn view<'a>(gb: &'a GameBananaState, status: &'a str) -> Element<'a, Message> {
    let filter_note = match gb.browse_category {
        GameBananaBrowseCategory::All => String::new(),
        c => format!(" · {}", c.label()),
    };
    let mode_label = match &gb.source {
        GameBananaListSource::Browse => {
            text(format!("Showing: newest Deadlock mods (browse){filter_note}")).size(13)
        }
        GameBananaListSource::Search(q) => {
            text(format!("Showing: search results for “{q}”{filter_note}")).size(13)
        }
    }
    .style(|theme| text::secondary(theme))
    .wrapping(text::Wrapping::Word);

    let category_row: Element<'a, Message> = row(
        GameBananaBrowseCategory::VARIANTS
            .iter()
            .map(|&cat| {
                let mut b = button(text(cat.label()).size(12))
                    .padding([6, 12])
                    .on_press(Message::GameBananaBrowseCategorySelected(cat));
                b = if gb.browse_category == cat {
                    b.style(|theme, status| button::primary(theme, status))
                } else {
                    b.style(|theme, status| button::secondary(theme, status))
                };
                b.into()
            }),
    )
    .spacing(8)
    .align_y(alignment::Vertical::Center)
    .into();

    let back = button(text("←  Mod library").size(14))
        .padding([10, 16])
        .on_press(Message::GameBananaBack)
        .style(|theme, status| button::secondary(theme, status));

    let title = text("GameBanana — Deadlock")
        .size(26)
        .font(Font {
            weight: Weight::Semibold,
            ..Font::DEFAULT
        });

    let search_field = text_input("Search Deadlock mods (type 2+ characters)", &gb.search_draft)
        .on_input(Message::GameBananaSearchInput)
        .on_submit(Message::GameBananaSearchSubmit)
        .padding([10, 14])
        .size(14)
        .width(Fill);

    let search_btn = button(text("Search").size(14))
        .padding([10, 18])
        .on_press(Message::GameBananaSearchSubmit)
        .style(|theme, status| button::secondary(theme, status));

    let browse_btn = button(text("Browse all").size(14))
        .padding([10, 18])
        .on_press(Message::GameBananaBrowseMode)
        .style(|theme, status| button::secondary(theme, status));

    let mp = max_page(gb.total_count, gb.per_page);
    let pager = row![
        button(text("← Prev").size(13))
            .padding([8, 14])
            .on_press_maybe((gb.page > 1).then(|| Message::GameBananaPagePrev))
            .style(|theme, status| button::secondary(theme, status)),
        text(format!(
            "Page {} of {}  ·  {} mods indexed on GameBanana",
            gb.page,
            mp,
            gb.total_count
        ))
        .size(13)
        .style(|theme| text::secondary(theme)),
        button(text("Next →").size(13))
            .padding([8, 14])
            .on_press_maybe(((gb.page as u64) < mp).then(|| Message::GameBananaPageNext))
            .style(|theme, status| button::secondary(theme, status)),
    ]
    .spacing(14)
    .align_y(alignment::Vertical::Center);

    let list_body: Element<'a, Message> = if gb.list_loading {
        text("Loading list…")
            .size(14)
            .style(|theme| text::secondary(theme))
            .into()
    } else if let Some(err) = &gb.list_error {
        text(format!("{err}"))
            .size(13)
            .style(|theme| text::secondary(theme))
            .wrapping(text::Wrapping::Word)
            .into()
    } else if gb.mods.is_empty() {
        text("No mods on this page.")
            .size(14)
            .style(|theme| text::secondary(theme))
            .into()
    } else {
        scrollable(
            gb.mods.iter().fold(column![].spacing(10), |col, m| {
                let sel = gb.selected_mod_id == Some(m.id);
                col.push(mod_row(m, gb, sel))
            }),
        )
        .height(Fill)
        .into()
    };

    let list_panel = container(
        column![
            text("Mods").size(16).font(Font {
                weight: Weight::Semibold,
                ..Font::DEFAULT
            }),
            list_body,
        ]
        .spacing(10),
    )
    .width(Fill)
    .height(Fill)
    .padding(14)
    .style(ui::surface_card);

    let detail_header: Element<'a, Message> = if let Some(name) = &gb.selected_mod_name {
        let link: Element<'a, Message> = if let Some(id) = gb.selected_mod_id {
            let url = format!("https://gamebanana.com/mods/{id}");
            button(text("Open on gamebanana.com").size(12))
                .padding([6, 10])
                .on_press(Message::GameBananaOpenModUrl(url))
                .style(|theme, status| button::text(theme, status))
                .into()
        } else {
            column![].into()
        };

        column![
            text(name).size(17).font(Font {
                weight: Weight::Semibold,
                ..Font::DEFAULT
            }),
            link,
        ]
        .spacing(6)
        .into()
    } else {
        column![text("Select a mod from the list.").size(14)].into()
    };

    let files_block: Element<'a, Message> = if gb.files_loading {
        text("Loading downloads…")
            .size(13)
            .style(|theme| text::secondary(theme))
            .into()
    } else if let Some(err) = &gb.files_error {
        text(format!("{err}"))
            .size(12)
            .style(|theme| text::secondary(theme))
            .wrapping(text::Wrapping::Word)
            .into()
    } else if gb.files.is_empty() {
        text("No file rows returned (this mod may have no packaged files).")
            .size(12)
            .style(|theme| text::secondary(theme))
            .wrapping(text::Wrapping::Word)
            .into()
    } else {
        scrollable(
            gb.files
                .iter()
                .fold(column![].spacing(8), |col, f| col.push(file_row(f, gb.import_busy))),
        )
        .height(Fill)
        .into()
    };

    let detail_carousel = detail_preview_carousel(gb);

    let detail_panel = container(
        column![
            detail_header,
            detail_carousel,
            text("Downloads")
                .size(14)
                .font(Font {
                    weight: Weight::Semibold,
                    ..Font::DEFAULT
                }),
            text(format!(
                "GameBanana game id {}. .vpk, .zip, and .rar (containing a .vpk) import automatically.",
                gamebanana::DEADLOCK_GAME_ID
            ))
            .size(11)
            .style(|theme| text::secondary(theme))
            .wrapping(text::Wrapping::Word),
            files_block,
        ]
        .spacing(10),
    )
    .width(Fill)
    .height(Fill)
    .padding(14)
    .style(ui::surface_card);

    let main_row = row![list_panel.width(Fill), detail_panel.width(Fill)]
        .spacing(16)
        .height(Fill);

    let status_line = text(status)
        .size(12)
        .style(|theme| text::secondary(theme))
        .wrapping(text::Wrapping::Word);

    let status_bar = container(status_line)
        .padding([10, 14])
        .width(Fill)
        .style(ui::status_footer);

    let body = container(
        column![main_row, status_bar]
            .spacing(14)
            .height(Fill)
            .max_width(ui::CONTENT_MAX_WIDTH),
    )
    .width(Fill)
    .height(Fill)
    .align_x(alignment::Horizontal::Center)
    .padding(padding::top(16).bottom(24).left(32).right(32));

    let nav = container(
        column![
            row![back, title].spacing(16).align_y(alignment::Vertical::Center),
            mode_label,
            category_row,
            row![search_field, search_btn, browse_btn]
                .spacing(10)
                .align_y(alignment::Vertical::Center),
            pager,
        ]
        .spacing(10),
    )
    .width(Fill)
    .padding([20, 32])
    .style(ui::nav_bar);

    let divider = rule::horizontal(1).style(rule::weak);

    container(column![nav, divider, body].spacing(0))
        .width(Fill)
        .height(Fill)
        .style(ui::page_backdrop)
        .into()
}
