use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill};

use crate::message::Message;
use crate::mod_manager::ModManager;

#[derive(Debug)]
pub struct App {
    mod_manager: ModManager,
    status: String,
}

impl Default for App {
    fn default() -> Self {
        let mut app = Self {
            mod_manager: ModManager::new(),
            status: String::new(),
        };

        match ModManager::mods_dir() {
            Ok(path) => {
                app.status = format!("Mods folder: {}", path.display());
            }
            Err(error) => {
                app.status = error;
            }
        }

        app
    }
}

impl App {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::UploadMod => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("Select a mod file")
                    .pick_file()
                {
                    match self.mod_manager.import_file(&path) {
                        Ok(saved_path) => {
                            self.status = format!("Imported to {}", saved_path.display());
                        }
                        Err(error) => {
                            self.status = format!("Import failed: {error}");
                        }
                    }
                } else {
                    self.status = String::from("File selection cancelled");
                }
            }

            Message::RemoveMod(index) => match self.mod_manager.remove_mod(index) {
                Ok(()) => {
                    self.status = String::from("Mod removed");
                }
                Err(error) => {
                    self.status = format!("Remove failed: {error}");
                }
            },

            Message::RefreshMods => match self.mod_manager.refresh() {
                Ok(()) => {
                    self.status = String::from("Mod list refreshed");
                }
                Err(error) => {
                    self.status = format!("Refresh failed: {error}");
                }
            },
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let controls = row![
            button("Upload Mod").on_press(Message::UploadMod),
            button("Refresh").on_press(Message::RefreshMods),
        ]
            .spacing(10);

        let mod_list = if self.mod_manager.mods.is_empty() {
            column![text("No mods installed")]
        } else {
            self.mod_manager.mods.iter().enumerate().fold(
                column![].spacing(10),
                |column, (index, mod_file)| {
                    column.push(
                        row![
                            text(&mod_file.file_name).width(Fill),
                            button("Remove").on_press(Message::RemoveMod(index)),
                        ]
                            .spacing(10),
                    )
                },
            )
        };

        container(
            column![
                text("Mod Manager").size(32),
                controls,
                text(&self.status),
                scrollable(mod_list).height(Fill),
            ]
                .spacing(20)
                .padding(20),
        )
            .width(Fill)
            .height(Fill)
            .into()
    }
}