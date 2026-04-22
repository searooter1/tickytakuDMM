use iced::widget::image::Handle;
use iced::Task;

use crate::gamebanana;
use crate::message::Message;
use crate::mod_manager::ModManager;
use crate::state::{
    AppState, GameBananaBrowseCategory, GameBananaListSource, GameBananaState, ModListState, Page,
};

fn sync_selected_preview_url(gb: &mut GameBananaState) {
    let Some(mid) = gb.selected_mod_id else {
        return;
    };
    let idx = *gb.thumb_carousel_index.get(&mid).unwrap_or(&0);
    gb.selected_preview_url = gb.mods.iter().find(|m| m.id == mid).and_then(|m| {
        m.preview_image_urls
            .get(idx)
            .cloned()
            .or_else(|| m.preview_image_urls.first().cloned())
    });
}

pub fn update(mod_manager: &mut ModManager, state: &mut AppState, message: Message) -> Task<Message> {
    let Page::GameBanana(gb) = &mut state.page else {
        return Task::none();
    };

    match message {
        Message::GameBananaBack => {
            state.page = Page::ModList(ModListState::default());
            state.status = String::from("Returned to your mod library.");
            Task::none()
        }

        Message::GameBananaSearchInput(text) => {
            gb.search_draft = text;
            Task::none()
        }

        Message::GameBananaBrowseCategorySelected(cat) => {
            if gb.browse_category == cat {
                return Task::none();
            }
            gb.browse_category = cat;
            gb.page = 1;
            gb.list_loading = true;
            gb.list_error = None;
            gb.mods.clear();
            gb.selected_mod_id = None;
            gb.selected_mod_name = None;
            gb.selected_preview_url = None;
            gb.files.clear();
            gb.files_error = None;
            gb.thumb_carousel_index.clear();
            gb.list_request_generation += 1;
            let list_gen = gb.list_request_generation;
            let cat_id = gb.browse_category.category_id();
            list_fetch_task(gb.source.clone(), gb.page, gb.per_page, list_gen, cat_id)
        }

        Message::GameBananaSearchSubmit => {
            let trimmed = gb.search_draft.trim();
            if trimmed.len() < 2 {
                state.status =
                    String::from("GameBanana search needs at least 2 characters in the box.");
                return Task::none();
            }
            gb.source = GameBananaListSource::Search(trimmed.to_string());
            gb.page = 1;
            gb.list_loading = true;
            gb.list_error = None;
            gb.mods.clear();
            gb.selected_mod_id = None;
            gb.selected_mod_name = None;
            gb.selected_preview_url = None;
            gb.files.clear();
            gb.files_error = None;
            gb.thumb_carousel_index.clear();
            gb.list_request_generation += 1;
            let list_gen = gb.list_request_generation;
            let cat_id = gb.browse_category.category_id();
            list_fetch_task(gb.source.clone(), gb.page, gb.per_page, list_gen, cat_id)
        }

        Message::GameBananaBrowseMode => {
            gb.source = GameBananaListSource::Browse;
            gb.page = 1;
            gb.list_loading = true;
            gb.list_error = None;
            gb.mods.clear();
            gb.selected_mod_id = None;
            gb.selected_mod_name = None;
            gb.selected_preview_url = None;
            gb.files.clear();
            gb.files_error = None;
            gb.thumb_carousel_index.clear();
            gb.list_request_generation += 1;
            let list_gen = gb.list_request_generation;
            let cat_id = gb.browse_category.category_id();
            list_fetch_task(gb.source.clone(), gb.page, gb.per_page, list_gen, cat_id)
        }

        Message::GameBananaPagePrev => {
            if gb.page <= 1 {
                return Task::none();
            }
            gb.page -= 1;
            gb.list_loading = true;
            gb.list_error = None;
            gb.list_request_generation += 1;
            let list_gen = gb.list_request_generation;
            let cat_id = gb.browse_category.category_id();
            list_fetch_task(gb.source.clone(), gb.page, gb.per_page, list_gen, cat_id)
        }

        Message::GameBananaPageNext => {
            let per = gb.per_page as u64;
            let max_page = if gb.total_count == 0 {
                1
            } else {
                (gb.total_count + per - 1) / per
            };
            if gb.page as u64 >= max_page {
                return Task::none();
            }
            gb.page += 1;
            gb.list_loading = true;
            gb.list_error = None;
            gb.list_request_generation += 1;
            let list_gen = gb.list_request_generation;
            let cat_id = gb.browse_category.category_id();
            list_fetch_task(gb.source.clone(), gb.page, gb.per_page, list_gen, cat_id)
        }

        Message::GameBananaListLoaded {
            generation,
            page,
            result,
        } => {
            if generation != gb.list_request_generation {
                return Task::none();
            }
            gb.list_loading = false;
            match result {
                Ok((mods, total)) => {
                    gb.thumbnails.clear();
                    gb.thumb_carousel_index.clear();
                    let grouped: Vec<_> = mods
                        .iter()
                        .map(|m| (m.id, m.preview_image_urls.clone()))
                        .filter(|(_, urls)| !urls.is_empty())
                        .collect();

                    gb.page = page;
                    gb.mods = mods;
                    gb.total_count = total;
                    gb.list_error = None;
                    state.status = format!(
                        "GameBanana: loaded {} Deadlock mods (page {page}).",
                        gb.mods.len()
                    );

                    let list_generation = generation;
                    return Task::perform(
                        async move {
                            let loaded = gamebanana::fetch_mod_thumbnails(grouped).await;
                            Message::GameBananaThumbnailsReady {
                                list_generation,
                                loaded,
                            }
                        },
                        std::convert::identity,
                    );
                }
                Err(e) => {
                    gb.thumbnails.clear();
                    gb.thumb_carousel_index.clear();
                    gb.list_error = Some(e.clone());
                    state.status = format!("GameBanana list failed: {e}");
                }
            }
            Task::none()
        }

        Message::GameBananaThumbnailsReady {
            list_generation,
            loaded,
        } => {
            if list_generation != gb.list_request_generation {
                return Task::none();
            }
            for (id, list) in loaded {
                let handles: Vec<Handle> = list.into_iter().map(Handle::from_bytes).collect();
                if !handles.is_empty() {
                    gb.thumbnails.insert(id, handles);
                    gb.thumb_carousel_index.entry(id).or_insert(0);
                }
            }
            Task::none()
        }

        Message::GameBananaThumbCarousel { mod_id, next } => {
            let Some(handles) = gb.thumbnails.get(&mod_id).filter(|h| !h.is_empty()) else {
                return Task::none();
            };
            let len = handles.len();
            let idx = gb.thumb_carousel_index.entry(mod_id).or_insert(0);
            if next {
                *idx = (*idx + 1) % len;
            } else {
                *idx = (*idx + len - 1) % len;
            }
            if gb.selected_mod_id == Some(mod_id) {
                sync_selected_preview_url(gb);
            }
            Task::none()
        }

        Message::GameBananaSelectMod(mod_id) => {
            gb.selected_mod_id = Some(mod_id);
            gb.selected_mod_name = gb
                .mods
                .iter()
                .find(|m| m.id == mod_id)
                .map(|m| m.name.clone());
            sync_selected_preview_url(gb);
            gb.files.clear();
            gb.files_loading = true;
            gb.files_error = None;
            gb.files_request_generation += 1;
            let files_gen = gb.files_request_generation;
            Task::perform(
                async move {
                    let result = gamebanana::fetch_mod_files(mod_id).await;
                    Message::GameBananaFilesLoaded {
                        generation: files_gen,
                        mod_id,
                        result,
                    }
                },
                std::convert::identity,
            )
        }

        Message::GameBananaFilesLoaded {
            generation,
            mod_id,
            result,
        } => {
            if generation != gb.files_request_generation || gb.selected_mod_id != Some(mod_id) {
                return Task::none();
            }
            gb.files_loading = false;
            match result {
                Ok((files, title)) => {
                    gb.selected_mod_name = Some(title);
                    gb.files = files;
                    gb.files_error = None;
                    state.status = String::from("Pick a download that contains a .vpk (ZIP is unpacked automatically).");
                }
                Err(e) => {
                    gb.files_error = Some(e.clone());
                    gb.files.clear();
                    state.status = format!("Could not load downloads: {e}");
                }
            }
            Task::none()
        }

        Message::GameBananaOpenModUrl(url) => {
            let _ = open::that(&url);
            Task::none()
        }

        Message::GameBananaDownloadFile(file) => {
            if gb.import_busy {
                return Task::none();
            }
            let Some(mod_name) = gb.selected_mod_name.clone() else {
                state.status = String::from("Select a mod first.");
                return Task::none();
            };
            let preview = gb.selected_preview_url.clone();
            gb.import_busy = true;
            state.status = format!("Downloading “{}”…", file.file_name);
            Task::perform(
                gamebanana::download_and_prepare_import(file, mod_name, preview),
                Message::GameBananaImportDone,
            )
        }

        Message::GameBananaImportDone(result) => {
            gb.import_busy = false;

            match result {
                Ok(payload) => {
                    let title = payload.title.clone();
                    let thumb = payload.thumbnail_path.as_deref();
                    let scratch = payload.scratch_dir.clone();
                    let res = mod_manager.import_file_with_metadata(
                        &payload.vpk_path,
                        payload.title,
                        None,
                        thumb,
                    );
                    let _ = std::fs::remove_dir_all(&scratch);
                    match res {
                        Ok(dest) => {
                            state.page = Page::ModList(ModListState::default());
                            state.status = format!(
                                "Imported from GameBanana into your library: {} ({})",
                                title,
                                dest.display()
                            );
                        }
                        Err(e) => {
                            state.status = format!("Import failed after download: {e}");
                        }
                    }
                }
                Err(e) => {
                    state.status = format!("Download / extract failed: {e}");
                }
            }
            Task::none()
        }

        _ => Task::none(),
    }
}

fn list_fetch_task(
    source: GameBananaListSource,
    page: u32,
    per_page: u32,
    generation: u64,
    category_id: Option<u32>,
) -> Task<Message> {
    Task::perform(
        async move {
            let result = match &source {
                GameBananaListSource::Browse => {
                    gamebanana::fetch_browse_page(page, per_page, category_id).await
                }
                GameBananaListSource::Search(q) => {
                    gamebanana::fetch_search_page(q.clone(), page, per_page, category_id).await
                }
            };
            Message::GameBananaListLoaded {
                generation,
                page,
                result,
            }
        },
        std::convert::identity,
    )
}

/// Initial fetch when entering the browser (generation must match `GameBananaState::new_browse`).
pub fn initial_list_task() -> Task<Message> {
    list_fetch_task(
        GameBananaListSource::Browse,
        1,
        20,
        1,
        GameBananaBrowseCategory::default().category_id(),
    )
}
