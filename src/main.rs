mod app;
mod message;
mod mod_file;
mod mod_manager;

pub fn main() -> iced::Result {
    iced::run(app::App::update, app::App::view)
}