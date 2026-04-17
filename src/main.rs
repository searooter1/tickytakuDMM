mod app;
mod components;
mod message;
mod mod_file;
mod mod_manager;
mod state;
mod update;
mod view;

pub fn main() -> iced::Result {
    iced::run(app::App::update, app::App::view)
}