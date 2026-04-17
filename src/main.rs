mod app;
mod components;
mod mod_file;
mod mod_manager;
mod pages;

pub fn main() -> iced::Result {
    iced::run(app::App::update, app::App::view)
}