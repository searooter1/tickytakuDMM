//! Patch Deadlock `gameinfo.gi` so VPKs under `citadel/addons` are loaded.
//! Steam updates can reset this file; we re-apply when enabling a mod.

use std::fs;
use std::path::Path;

/// Key / value pairs inside `SearchPaths { ... }`, in engine order.
const DESIRED_SEARCH_PATH_ENTRIES: [(&str, &str); 7] = [
    ("Game", "citadel/addons"),
    ("Mod", "citadel"),
    ("Write", "citadel"),
    ("Game", "citadel"),
    ("Mod", "core"),
    ("Write", "core"),
    ("Game", "core"),
];

fn desired_entries_owned() -> Vec<(String, String)> {
    DESIRED_SEARCH_PATH_ENTRIES
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect()
}

/// If `SearchPaths` already matches this layout, the file is left unchanged.
pub fn ensure_addon_search_paths(gameinfo_path: &Path) -> Result<(), String> {
    let original = fs::read_to_string(gameinfo_path).map_err(|e| {
        format!(
            "Could not read Deadlock gameinfo ({}): {e}",
            gameinfo_path.display()
        )
    })?;

    let Some((block_start, block_end, open_brace, close_brace)) = find_search_paths_block(&original)
    else {
        return Err(String::from(
            "Could not find a `SearchPaths { ... }` block in gameinfo.gi. Mods may not load until this is fixed manually.",
        ));
    };

    let inner = &original[open_brace + 1..close_brace];
    if parse_search_path_entries(inner) == desired_entries_owned() {
        return Ok(());
    }

    let indent_outer = line_leading_whitespace(&original, block_start);
    let indent_inner = format!("{indent_outer}\t");

    let mut inner_body = String::new();
    for (key, value) in &DESIRED_SEARCH_PATH_ENTRIES {
        inner_body.push_str(&format!("{indent_inner}{key}\t\t\t\t{value}\n"));
    }

    let new_block = format!(
        "{indent_outer}SearchPaths\n{indent_outer}{{\n{inner_body}{indent_outer}}}\n"
    );

    let mut out = String::with_capacity(
        original
            .len()
            .saturating_sub(block_end - block_start)
            .saturating_add(new_block.len()),
    );
    out.push_str(&original[..block_start]);
    out.push_str(&new_block);
    out.push_str(&original[block_end..]);

    let backup_path = gameinfo_path.with_extension("gi.tickytaku.bak");
    fs::copy(gameinfo_path, &backup_path).map_err(|e| {
        format!(
            "Could not back up gameinfo to {} before patching: {e}",
            backup_path.display()
        )
    })?;

    write_same_dir_replace(gameinfo_path, &out)?;

    Ok(())
}

fn parse_search_path_entries(inner: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in inner.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with("//") {
            continue;
        }
        let mut parts = t.split_whitespace();
        let Some(key) = parts.next() else {
            continue;
        };
        let value = parts.collect::<Vec<_>>().join(" ");
        if value.is_empty() {
            continue;
        }
        out.push((key.to_string(), value));
    }
    out
}

fn line_leading_whitespace(content: &str, line_start: usize) -> String {
    let rest = content.get(line_start..).unwrap_or("");
    let mut end = 0usize;
    for (i, ch) in rest.char_indices() {
        if ch == ' ' || ch == '\t' {
            end = i + ch.len_utf8();
            continue;
        }
        break;
    }
    rest.get(..end).unwrap_or("").to_string()
}

/// `(block_start, block_end_exclusive, open_brace, close_brace)` — `block_start` is the first byte
/// of the line containing `SearchPaths`; `block_end` is the byte after the closing `}`.
fn find_search_paths_block(content: &str) -> Option<(usize, usize, usize, usize)> {
    for (idx, _) in content.match_indices("SearchPaths") {
        let line_start = content[..idx].rfind('\n').map(|n| n + 1).unwrap_or(0);
        let line_end = content[idx..]
            .find('\n')
            .map(|n| idx + n)
            .unwrap_or(content.len());
        if content.get(line_start..line_end)?.trim() != "SearchPaths" {
            continue;
        }

        let bytes = content.as_bytes();
        let mut p = line_end;
        while p < bytes.len() && matches!(bytes[p], b' ' | b'\t' | b'\r' | b'\n') {
            p += 1;
        }
        if p >= bytes.len() || bytes[p] != b'{' {
            continue;
        }
        let open_brace = p;
        let close_brace = matching_close_brace(content, open_brace)?;
        return Some((line_start, close_brace + 1, open_brace, close_brace));
    }
    None
}

fn matching_close_brace(content: &str, open_brace: usize) -> Option<usize> {
    let mut depth = 0u32;
    for (i, ch) in content.get(open_brace..)?.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(open_brace + i);
                }
            }
            _ => {}
        }
    }
    None
}

fn write_same_dir_replace(path: &Path, content: &str) -> Result<(), String> {
    let tmp = path.with_extension("gi.tickytaku.tmp");
    fs::write(&tmp, content.as_bytes()).map_err(|e| {
        format!(
            "Could not write temp gameinfo ({}): {e}",
            tmp.display()
        )
    })?;

    if path.exists() {
        fs::remove_file(path).map_err(|e| {
            format!(
                "Could not replace gameinfo ({}); close Deadlock if it is running. ({e})",
                path.display()
            )
        })?;
    }

    fs::rename(&tmp, path).map_err(|e| {
        format!(
            "Could not install patched gameinfo ({}): {e}",
            path.display()
        )
    })?;

    Ok(())
}
