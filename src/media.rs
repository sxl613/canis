use std::fs;
use std::path::Path;
use std::time::SystemTime;
use std::{cmp::Reverse, path::PathBuf};

use walkdir::WalkDir;

use crate::{ListParams, SortField};

#[derive(Debug, Clone)]
pub struct MediaFile {
    pub name: String,
    pub path: String, // relative path
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub created: Option<SystemTime>,
    pub extension: String,
}

pub fn list_media_files(media_path: &Path, params: &ListParams) -> Vec<MediaFile> {
    let mut files = Vec::new();

    let valid_extensions = ["mp4", "webm", "mkv", "avi", "mov"];

    for entry in WalkDir::new(media_path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        if !valid_extensions.contains(&extension.as_str()) {
            continue;
        }

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let relative_path = path
            .strip_prefix(media_path)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        files.push(MediaFile {
            name,
            path: format!("/media/{}", relative_path),
            size: metadata.len(),
            modified: metadata.modified().ok(),
            created: metadata.created().ok(),
            extension,
        })
    }

    match params.sort {
        SortField::Name => files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
        SortField::Size => files.sort_by_key(|f| f.size),
        SortField::Created => files.sort_by(|a, b| a.created.cmp(&b.created)),
        SortField::LastModified => files.sort_by_key(|f| Reverse(f.modified)),
    };
    let page_size = (params.page_size as usize).max(1);
    let total_pages = (files.len() + page_size - 1) / page_size.max(1);
    let page = (params.page as usize).clamp(1, total_pages.max(1));
    let start = (page - 1) * page_size;
    let end = (start + page_size).min(files.len());
    files[start..end].to_vec()
}
pub fn find_media_file(media_path: &Path, filename: &String) -> Option<MediaFile> {
    let valid_extensions = ["mp4", "webm", "mkv", "avi", "mov"];

    // Canonicalize the base dir once
    let base = media_path.canonicalize().ok()?;

    // Optional “fast fail”: reject obvious path separators in the *input* name.
    // Canonicalize + starts_with is the real protection, but this improves ergonomics.
    if filename.contains('/') || filename.contains('\\') {
        return None;
    }

    let full_path: PathBuf = base.join(&filename);

    // canonicalize resolves ../ and follows symlinks; if file doesn't exist, this fails.
    let canonical = full_path.canonicalize().ok()?;

    // Prevent path escape (incl. via symlinks)
    if !canonical.starts_with(&base) {
        return None;
    }

    // Must be a file
    let metadata = fs::metadata(&canonical).ok()?;
    if !metadata.is_file() {
        return None;
    }

    // Extension allow-list (case-insensitive)
    let extension = canonical
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())?;
    if !valid_extensions.contains(&extension.as_str()) {
        return None;
    }

    // Relative path for URL building: canonical is inside base (checked above)
    let relative_path = canonical.strip_prefix(&base).ok()?;

    // Convert to a URL-ish path segment (handle Windows '\')
    let relative_url = relative_path.to_string_lossy().replace('\\', "/");

    let name = canonical
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    Some(MediaFile {
        name,
        path: format!("/media/{}", relative_url),
        size: metadata.len(),
        modified: metadata.modified().ok(),
        created: metadata.created().ok(),
        extension,
    })
}

pub fn format_size(bytes: &u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes.clone() as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.1} {}", size, UNITS[unit_idx])
}
