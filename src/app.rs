use std::path::PathBuf;

use iced::widget::{button, column, container, image, row, scrollable, text, text_input};
use iced::{Element, Fill, Length};

use crate::message::Message;
use crate::mod_manager::ModManager;

#[derive(Debug, Clone)]
struct ImportForm {
    mod_path: PathBuf,
    title: String,
    description: String,
    thumbnail_path: Option<PathBuf>,
}

#[derive(Debug)]
pub struct App {
    mod_manager: ModManager,
    status: String,
    import_form: Option<ImportForm>,
}

impl Default for App {
    fn default() -> Self {
        let mut app = Self {
            mod_manager: ModManager::new(),
            status: String::new(),
            import_form: None,
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
            Message::StartUploadMod => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("Select a mod file")
                    .add_filter("Deadlock mod package", &["vpk"])
                    .pick_file()
                {
                    let default_title = path
                        .file_stem()
                        .and_then(|name| name.to_str())
                        .unwrap_or("New Mod")
                        .to_string();

                    self.import_form = Some(ImportForm {
                        mod_path: path,
                        title: default_title,
                        description: String::new(),
                        thumbnail_path: None,
                    });

                    self.status = String::from("Fill out the mod details, then save.");
                } else {
                    self.status = String::from("File selection cancelled");
                }
            }

            Message::ImportTitleChanged(value) => {
                if let Some(form) = &mut self.import_form {
                    form.title = value;
                }
            }

            Message::ImportDescriptionChanged(value) => {
                if let Some(form) = &mut self.import_form {
                    form.description = value;
                }
            }

            Message::PickThumbnail => {
                if let Some(form) = &mut self.import_form {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_title("Select a thumbnail image")
                        .add_filter("Image", &["png", "jpg", "jpeg", "webp"])
                        .pick_file()
                    {
                        form.thumbnail_path = Some(path.clone());
                        self.status = format!("Selected thumbnail: {}", path.display());
                    } else {
                        self.status = String::from("Thumbnail selection cancelled");
                    }
                }
            }

            Message::SaveImport => {
                let Some(form) = self.import_form.clone() else {
                    self.status = String::from("No import form is open");
                    return;
                };

                let trimmed_title = form.title.trim().to_string();

                if trimmed_title.is_empty() {
                    self.status = String::from("Title is required");
                    return;
                }

                let description = {
                    let trimmed = form.description.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_string())
                    }
                };

                match self.mod_manager.import_file_with_metadata(
                    &form.mod_path,
                    trimmed_title,
                    description,
                    form.thumbnail_path.as_deref(),
                ) {
                    Ok(saved_path) => {
                        self.import_form = None;
                        self.status = format!("Imported to {}", saved_path.display());
                    }
                    Err(error) => {
                        self.status = format!("Import failed: {error}");
                    }
                }
            }

            Message::CancelImport => {
                self.import_form = None;
                self.status = String::from("Import cancelled");
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
        if let Some(form) = &self.import_form {
            return self.import_form_view(form);
        }

        self.main_view()
    }

    fn main_view(&self) -> Element<'_, Message> {
        let controls = row![
            button("Upload Mod").on_press(Message::StartUploadMod),
            button("Refresh").on_press(Message::RefreshMods),
        ]
            .spacing(10);

        let mod_list = if self.mod_manager.mods.is_empty() {
            column![text("No mods installed")]
        } else {
            self.mod_manager.mods.iter().enumerate().fold(
                column![].spacing(15),
                |column, (index, mod_file)| {
                    let description_text = mod_file
                        .description
                        .as_deref()
                        .unwrap_or("No description");

                    let thumbnail_widget: Element<'_, Message> = if let Some(path) = &mod_file.thumbnail_path {
                        image(path.clone())
                            .width(Length::Fixed(96.0))
                            .height(Length::Fixed(96.0))
                            .into()
                    } else {
                        text("No thumbnail").size(14).into()
                    };

                    column.push(
                        container(
                            row![
                                thumbnail_widget,
                                column![
                                    row![
                                        text(&mod_file.title).size(22).width(Fill),
                                        button("Remove").on_press(Message::RemoveMod(index)),
                                    ]
                                    .spacing(10),
                                    text(format!("File: {}", mod_file.file_name)).size(14),
                                    text(format!("Description: {description_text}")).size(14),
                                ]
                                .spacing(6)
                                .width(Fill),
                            ].spacing(12),
                        ).padding(12),
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

    fn import_form_view(&self, form: &ImportForm) -> Element<'_, Message> {
        let thumbnail_info = form
            .thumbnail_path
            .as_ref()
            .map(|path| format!("Selected thumbnail: {}", path.display()))
            .unwrap_or_else(|| String::from("No thumbnail selected"));

        let thumbnail_preview: Element<'_, Message> = if let Some(path) = &form.thumbnail_path {
            image(path.clone())
                .width(Length::Fixed(120.0))
                .height(Length::Fixed(120.0))
                .into()
        } else {
            text("No thumbnail selected").size(14).into()
        };

        container(
            column![
                text("Import Mod").size(32),
                text(format!("Selected file: {}", form.mod_path.display())).size(14),
                text("Title"),
                text_input("Enter a title", &form.title)
                    .on_input(Message::ImportTitleChanged),
                text("Description (optional)"),
                text_input("Enter a description", &form.description)
                    .on_input(Message::ImportDescriptionChanged),
                thumbnail_preview,
                text(thumbnail_info).size(14),
                row![
                    button("Choose Thumbnail").on_press(Message::PickThumbnail),
                    button("Save Mod").on_press(Message::SaveImport),
                    button("Cancel").on_press(Message::CancelImport),
                ]
                .spacing(10),
                text(&self.status),
            ]
                .spacing(16)
                .padding(20),
        )
            .width(Fill)
            .height(Fill)
            .into()
    }
}