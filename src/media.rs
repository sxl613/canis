use std::path::Path;

use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct MediaFile {
    pub name: String,
    pub path: String, // relative path
    pub size: u64,
    pub extension: String,
}

pub fn list_media_files(media_path: &Path) -> Vec<MediaFile> {
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
            extension,
        })
    }

    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    files
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
