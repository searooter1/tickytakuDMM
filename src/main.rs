mod app;
mod app_db;
mod components;
mod gamebanana;
mod gameinfo_gi;
mod message;
mod ui;
mod mod_file;
mod mod_manager;
mod state;
mod update;
mod view;

/// Tokyo Night look with green accents; `primary` and `success` share the same green so
/// primary-style and success-style buttons (e.g. Enable) match.
fn application_theme(_state: &app::App) -> iced::Theme {
    use iced::color;
    use iced::theme::Palette;

    const ACCENT_GREEN: iced::Color = color!(0x6eb887);

    iced::Theme::custom(
        "Tokyo Night (green primary)",
        Palette {
            primary: ACCENT_GREEN,
            success: ACCENT_GREEN,
            ..Palette::TOKYO_NIGHT
        },
    )
}

pub fn main() -> iced::Result {
    iced::application(app::App::default, app::App::update, app::App::view)
        .title("Tickytaku Mod Manager")
        .theme(application_theme)
        .window_size((1100, 720))
        .antialiasing(true)
        .centered()
        .run()
}
