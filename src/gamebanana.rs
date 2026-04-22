//! GameBanana HTTP helpers: browse/search Deadlock mods and download archives / VPKs.

use std::io::Cursor;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;
use std::fs::File;
use std::io::BufWriter;
use unarc_rs::unified::ArchiveFormat;

const USER_AGENT: &str = "tickytakuDMM/0.1 (Deadlock mod manager; +https://github.com/tickytaku)";
pub const DEADLOCK_GAME_ID: &str = "20948";

/// Max preview screenshots to fetch per mod (GameBanana pages can list many images).
const MAX_PREVIEW_IMAGES_PER_MOD: usize = 16;

#[derive(Debug, Clone)]
pub struct ModSummary {
    pub id: u64,
    pub name: String,
    pub has_files: bool,
    /// GameBanana screenshot URLs (ordered). Used for carousel + import thumbnail.
    pub preview_image_urls: Vec<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FileEntry {
    pub id: u64,
    pub file_name: String,
    pub download_url: String,
    pub size_bytes: u64,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ImportPayload {
    pub vpk_path: PathBuf,
    pub title: String,
    pub thumbnail_path: Option<PathBuf>,
    /// Entire temp tree (delete with `remove_dir_all` after import).
    pub scratch_dir: PathBuf,
}

fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|e| format!("HTTP client: {e}"))
}

fn preferred_preview_file(image: &Value) -> Option<&str> {
    // Prefer larger previews for on-screen carousels; fall back to smaller assets.
    image
        .get("_sFile530")
        .and_then(|v| v.as_str())
        .or_else(|| image.get("_sFile220").and_then(|v| v.as_str()))
        .or_else(|| image.get("_sFile100").and_then(|v| v.as_str()))
        .or_else(|| image.get("_sFile").and_then(|v| v.as_str()))
}

fn preview_image_urls_from_record(record: &Value) -> Vec<String> {
    let Some(images) = record
        .pointer("/_aPreviewMedia/_aImages")
        .and_then(|v| v.as_array())
    else {
        return Vec::new();
    };

    let mut urls = Vec::new();
    for image in images.iter().take(MAX_PREVIEW_IMAGES_PER_MOD) {
        let Some(base) = image.get("_sBaseUrl").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(file) = preferred_preview_file(image) else {
            continue;
        };
        urls.push(format!(
            "{}/{}",
            base.trim_end_matches('/'),
            file.trim_start_matches('/')
        ));
    }
    urls
}

fn looks_like_raster_image(bytes: &[u8]) -> bool {
    if bytes.len() < 12 {
        return false;
    }
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return true;
    }
    if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        return true;
    }
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return true;
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return true;
    }
    false
}

/// Download all preview screenshots for each mod (URLs from [`preview_image_urls_from_record`]).
pub async fn fetch_mod_thumbnails(grouped: Vec<(u64, Vec<String>)>) -> Vec<(u64, Vec<Vec<u8>>)> {
    let Ok(client) = client() else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for (id, urls) in grouped {
        if urls.is_empty() {
            continue;
        }
        let mut images = Vec::new();
        for url in urls {
            let Ok(response) = client.get(&url).send().await else {
                continue;
            };
            if !response.status().is_success() {
                continue;
            }
            let Ok(bytes) = response.bytes().await else {
                continue;
            };
            let slice = bytes.as_ref();
            if looks_like_raster_image(slice) {
                images.push(slice.to_vec());
            }
        }
        if !images.is_empty() {
            out.push((id, images));
        }
    }
    out
}

fn record_to_summary(record: &Value) -> Option<ModSummary> {
    let id = record.get("_idRow")?.as_u64()?;
    let name = record.get("_sName")?.as_str()?.to_string();
    let has_files = record.get("_bHasFiles").and_then(|v| v.as_bool()).unwrap_or(false);
    let preview_image_urls = preview_image_urls_from_record(record);
    Some(ModSummary {
        id,
        name,
        has_files,
        preview_image_urls,
    })
}

#[derive(Debug, Deserialize)]
struct ListEnvelope {
    #[serde(rename = "_aMetadata")]
    metadata: ListMetadata,
    #[serde(rename = "_aRecords")]
    records: Vec<Value>,
}

#[derive(Debug, Deserialize)]
struct ListMetadata {
    #[serde(rename = "_nRecordCount")]
    record_count: u64,
}

pub async fn fetch_browse_page(
    page: u32,
    per_page: u32,
    category_id: Option<u32>,
) -> Result<(Vec<ModSummary>, u64), String> {
    let client = client()?;
    let mut url = format!(
        "https://gamebanana.com/apiv10/Mod/Index?_nPage={page}&_nPerpage={per_page}&_aFilters%5BGeneric_Game%5D={}",
        DEADLOCK_GAME_ID
    );
    if let Some(id) = category_id {
        url.push_str(&format!("&_aFilters%5BGeneric_Category%5D={id}"));
    }
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("GameBanana request failed: {e}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "GameBanana returned HTTP {}",
            response.status()
        ));
    }
    let env: ListEnvelope = response
        .json()
        .await
        .map_err(|e| format!("Could not read mod list JSON: {e}"))?;
    let total = env.metadata.record_count;
    let mods = env
        .records
        .iter()
        .filter_map(|r| record_to_summary(r))
        .collect();
    Ok((mods, total))
}

pub async fn fetch_search_page(
    query: String,
    page: u32,
    per_page: u32,
    category_id: Option<u32>,
) -> Result<(Vec<ModSummary>, u64), String> {
    let trimmed = query.trim();
    if trimmed.len() < 2 {
        return Err(String::from("Search needs at least 2 characters."));
    }

    let client = client()?;
    let mut url = format!(
        "https://gamebanana.com/apiv10/Util/Search/Results?_sSearchString={}&_sModelName=Mod&_idGameRow={}&_nPage={}&_nPerpage={}",
        urlencoding_encode(trimmed),
        DEADLOCK_GAME_ID,
        page,
        per_page
    );
    if let Some(id) = category_id {
        url.push_str(&format!("&_aFilters%5BGeneric_Category%5D={id}"));
    }
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("GameBanana search failed: {e}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "GameBanana search returned HTTP {}",
            response.status()
        ));
    }
    let body: Value = response
        .json()
        .await
        .map_err(|e| format!("Could not read search JSON: {e}"))?;

    if let Some(code) = body.get("_sErrorCode").and_then(|v| v.as_str()) {
        let msg = body
            .pointer("/_aErrorData/_sSearchString/_sErrorMessage")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown API error");
        return Err(format!("GameBanana: {code} — {msg}"));
    }

    let env: ListEnvelope = serde_json::from_value(body)
        .map_err(|e| format!("Unexpected search response: {e}"))?;
    let total = env.metadata.record_count;
    let mods = env
        .records
        .iter()
        .filter_map(|r| record_to_summary(r))
        .collect();
    Ok((mods, total))
}

fn urlencoding_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.as_bytes() {
        match *b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(char::from(*b))
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

pub async fn fetch_mod_files(mod_id: u64) -> Result<(Vec<FileEntry>, String), String> {
    let client = client()?;
    let url = format!(
        "https://api.gamebanana.com/Core/Item/Data?itemtype=Mod&itemid={mod_id}&fields=Files().aFiles(),name"
    );
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("GameBanana file metadata request failed: {e}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "GameBanana file API returned HTTP {}",
            response.status()
        ));
    }
    let body: Value = response
        .json()
        .await
        .map_err(|e| format!("Could not read file metadata JSON: {e}"))?;

    let arr = body
        .as_array()
        .ok_or_else(|| String::from("Unexpected file metadata shape"))?;

    let files_value = arr
        .first()
        .ok_or_else(|| String::from("Empty file metadata response"))?;

    let mod_name = arr
        .get(1)
        .and_then(|v| v.as_str())
        .unwrap_or("Downloaded mod")
        .to_string();

    let mut files = Vec::new();
    if let Some(map) = files_value.as_object() {
        for (_key, entry) in map {
            let Some(id) = entry.get("_idRow").and_then(|v| v.as_u64()) else {
                continue;
            };
            let file_name = entry
                .get("_sFile")
                .and_then(|v| v.as_str())
                .unwrap_or("download")
                .to_string();
            let download_url = entry
                .get("_sDownloadUrl")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if download_url.is_empty() {
                continue;
            }
            let size_bytes = entry
                .get("_nFilesize")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let description = entry
                .get("_sDescription")
                .and_then(|v| v.as_str())
                .map(str::to_string);
            files.push(FileEntry {
                id,
                file_name,
                download_url,
                size_bytes,
                description,
            });
        }
    }

    files.sort_by(|a, b| {
        let rank = |f: &FileEntry| {
            let n = f.file_name.to_ascii_lowercase();
            if n.ends_with(".vpk") {
                0
            } else if n.ends_with(".zip") || n.ends_with(".rar") {
                1
            } else {
                2
            }
        };
        rank(a)
            .cmp(&rank(b))
            .then_with(|| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()))
    });

    Ok((files, mod_name))
}

fn is_zip(bytes: &[u8]) -> bool {
    bytes.len() >= 4 && bytes[0] == 0x50 && bytes[1] == 0x4b
}

fn is_rar(bytes: &[u8]) -> bool {
    // RAR 4.x / 5.x signature: "Rar!" + 0x1A + 0x07 (+ optional 0x01 for RAR5)
    bytes.len() >= 7
        && bytes[0..4] == *b"Rar!"
        && bytes[4] == 0x1a
        && bytes[5] == 0x07
}

fn unrar_extract_to_dir(archive_path: &Path, dest: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dest).map_err(|e| format!("Could not create extract folder: {e}"))?;

    let mut archive = ArchiveFormat::open_path(archive_path)
        .map_err(|e| format!("Could not open RAR archive: {e}"))?;

    while let Some(entry) = archive
        .next_entry()
        .map_err(|e| format!("RAR entry: {e:?}"))?
    {
        let name = entry.name().trim_start_matches(['/', '\\']);
        if name.is_empty() {
            continue;
        }
        let rel = Path::new(name);
        if name.ends_with('/') || name.ends_with('\\') {
            if let Some(outdir) = safe_join_under(dest, rel) {
                std::fs::create_dir_all(&outdir).map_err(|e| format!("{e}"))?;
            }
            continue;
        }
        let Some(outpath) = safe_join_under(dest, rel) else {
            return Err(format!("Unsafe path in RAR: {}", entry.name()));
        };
        if let Some(parent) = outpath.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{e}"))?;
        }
        let mut out = BufWriter::new(
            File::create(&outpath).map_err(|e| format!("Could not create {}: {e}", outpath.display()))?,
        );
        archive
            .read_to(&entry, &mut out)
            .map_err(|e| format!("Extract {}: {e}", entry.name()))?;
    }

    Ok(())
}

fn collect_vpks(root: &Path, out: &mut Vec<PathBuf>, depth: usize) -> Result<(), String> {
    if depth > 8 {
        return Ok(());
    }
    let read = std::fs::read_dir(root).map_err(|e| format!("Could not read {}: {e}", root.display()))?;
    for entry in read.flatten() {
        let path = entry.path();
        let meta = entry.metadata().map_err(|e| e.to_string())?;
        if meta.is_dir() {
            collect_vpks(&path, out, depth + 1)?;
        } else if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("vpk")) {
            out.push(path);
        }
    }
    Ok(())
}

fn first_vpk_under(dir: &Path) -> Result<PathBuf, String> {
    let mut found = Vec::new();
    collect_vpks(dir, &mut found, 0)?;
    found.sort();
    found.into_iter().next().ok_or_else(|| {
        String::from(
            "No .vpk found inside the archive. This mod may use a different layout; try downloading manually.",
        )
    })
}

fn unzip_to_dir(bytes: &[u8], dest: &Path) -> Result<(), String> {
    let reader = Cursor::new(bytes);
    let mut archive =
        zip::ZipArchive::new(reader).map_err(|e| format!("Could not read ZIP archive: {e}"))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("ZIP entry {i}: {e}"))?;
        let rel = Path::new(file.name());
        let Some(outpath) = safe_join_under(dest, rel) else {
            continue;
        };

        if file.name().ends_with('/') || file.is_dir() {
            std::fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut outfile = std::fs::File::create(&outpath).map_err(|e| e.to_string())?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Extract {}: {e}", outpath.display()))?;
        }
    }
    Ok(())
}

fn safe_join_under(base: &Path, rel: &Path) -> Option<PathBuf> {
    if rel.is_absolute() {
        return None;
    }
    if rel
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir | std::path::Component::RootDir))
    {
        return None;
    }
    Some(base.join(rel))
}

pub async fn download_and_prepare_import(
    file: FileEntry,
    mod_title: String,
    preview_url: Option<String>,
) -> Result<ImportPayload, String> {
    let client = client()?;
    let response = client
        .get(&file.download_url)
        .send()
        .await
        .map_err(|e| format!("Download failed: {e}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "Download returned HTTP {}",
            response.status()
        ));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Could not read download body: {e}"))?;
    let slice = bytes.as_ref();

    let work = tempfile::tempdir().map_err(|e| format!("Temp folder: {e}"))?;
    let work_path = work.path().to_path_buf();

    let lower = file.file_name.to_ascii_lowercase();
    let vpk_path = if lower.ends_with(".vpk") {
        let dest = work_path.join(&file.file_name);
        std::fs::write(&dest, slice).map_err(|e| format!("Could not save .vpk: {e}"))?;
        dest
    } else if lower.ends_with(".rar") || is_rar(slice) {
        let rar_path = work_path.join("download.rar");
        std::fs::write(&rar_path, slice).map_err(|e| format!("Could not save .rar: {e}"))?;
        let extract_dir = work_path.join("_rar_extract");
        unrar_extract_to_dir(&rar_path, &extract_dir)?;
        first_vpk_under(&extract_dir)?
    } else if lower.ends_with(".zip") && is_zip(slice) {
        unzip_to_dir(slice, &work_path)?;
        first_vpk_under(&work_path)?
    } else if is_zip(slice) {
        unzip_to_dir(slice, &work_path)?;
        first_vpk_under(&work_path)?
    } else {
        return Err(format!(
            "Unsupported file type “{}”. Use a .vpk, .zip, or .rar that contains a .vpk.",
            file.file_name
        ));
    };

    let thumbnail_path = if let Some(url) = preview_url {
        download_preview_to_dir(&client, &url, &work_path)
            .await
            .ok()
    } else {
        None
    };

    std::mem::forget(work);

    Ok(ImportPayload {
        vpk_path,
        title: mod_title,
        thumbnail_path,
        scratch_dir: work_path,
    })
}

async fn download_preview_to_dir(
    client: &reqwest::Client,
    url: &str,
    dir: &Path,
) -> Result<PathBuf, String> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Thumbnail download: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("Thumbnail HTTP {}", response.status()));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Thumbnail body: {e}"))?;
    let ext = if url.to_ascii_lowercase().contains(".png") {
        "png"
    } else {
        "jpg"
    };
    let path = dir.join(format!("gb_preview.{ext}"));
    std::fs::write(&path, &bytes).map_err(|e| e.to_string())?;
    Ok(path)
}
