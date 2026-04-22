//! Layout and container styles: dark-first shell (nav strip, cards, status).
use iced::border::{self, Border};
use iced::widget::container;
use iced::{Color, Shadow, Theme, Vector};

/// Main content column width (centered on large windows).
pub const CONTENT_MAX_WIDTH: f32 = 960.0;

/// Window canvas: slightly below cards so panels read clearly.
pub fn page_backdrop(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(palette.background.base.color.into()),
        ..Default::default()
    }
}

/// Full-width top bar (title + actions).
pub fn nav_bar(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(palette.background.weak.color.into()),
        text_color: Some(palette.background.weak.text),
        border: Border {
            width: 0.0,
            color: palette.background.strong.color,
            ..Default::default()
        },
        shadow: Shadow {
            color: Color::from_rgba8(0, 0, 0, 0.35),
            offset: Vector::new(0.0, 6.0),
            blur_radius: 18.0,
        },
        ..Default::default()
    }
}

/// Raised panels (mod rows, forms).
pub fn surface_card(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(palette.background.weak.color.into()),
        text_color: Some(palette.background.weak.text),
        border: Border {
            width: 1.0,
            color: palette.background.strong.color,
            ..border::rounded(16)
        },
        shadow: Shadow {
            color: Color::from_rgba8(0, 0, 0, 0.4),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 22.0,
        },
        ..Default::default()
    }
}

/// Thumbnail / preview well.
pub fn media_frame(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(palette.background.weakest.color.into()),
        text_color: Some(palette.background.weakest.text),
        border: Border {
            width: 1.0,
            color: palette.background.strong.color,
            ..border::rounded(12)
        },
        ..Default::default()
    }
}

/// Status / feedback line at the bottom of the list.
pub fn status_footer(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(palette.background.weakest.color.into()),
        text_color: Some(palette.background.weakest.text),
        border: Border {
            width: 1.0,
            color: palette.background.strong.color,
            ..border::rounded(10)
        },
        ..Default::default()
    }
}
