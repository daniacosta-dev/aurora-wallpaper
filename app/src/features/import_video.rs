use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::domain::{WallpaperId, WallpaperLibrary};
use crate::infrastructure::LibraryStorage;
use crate::features::thumbnail::extract_thumbnail;

pub const SUPPORTED_EXTENSIONS: &[&str] = &["mp4", "webm", "mov", "mkv"];

#[derive(Debug)]
pub enum ImportError {
    UnsupportedFormat(String),
    FileNotFound(PathBuf),
    AlreadyImported(PathBuf),
    Storage(crate::infrastructure::StorageError),
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportError::UnsupportedFormat(ext) => write!(f, "Unsupported video format: .{ext}"),
            ImportError::FileNotFound(p) => write!(f, "File not found: {}", p.display()),
            ImportError::AlreadyImported(p) => write!(
                f,
                "Already imported: {}",
                p.file_name().unwrap_or_default().to_string_lossy()
            ),
            ImportError::Storage(e) => write!(f, "Storage error: {e}"),
        }
    }
}

pub fn import_video(
    path: PathBuf,
    library: &mut WallpaperLibrary,
    storage: &LibraryStorage,
) -> Result<WallpaperId, ImportError> {
    // 1. File must exist.
    if !path.exists() {
        return Err(ImportError::FileNotFound(path));
    }

    // 2. Validate extension.
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if !SUPPORTED_EXTENSIONS.contains(&ext.as_str()) {
        return Err(ImportError::UnsupportedFormat(ext));
    }

    // 3. Prevent duplicates.
    if library.all().iter().any(|w| w.file_path == path) {
        return Err(ImportError::AlreadyImported(path));
    }

    // 4. Derive display name.
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .to_string();

    // 5. Timestamp.
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // 6. Add to library.
    let id = library.add(name, path.clone(), created_at);

    // 7. Generate thumbnail — non-fatal on failure.
    let thumbnails_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("aurorawall")
        .join("thumbnails");

    match extract_thumbnail(&path, &thumbnails_dir, id) {
        Ok(thumb_path) => {
            if let Some(w) = library.get_mut(id) {
                w.thumbnail_path = Some(thumb_path);
            }
        }
        Err(e) => eprintln!("[AuroraWall] Thumbnail warning: {e}"),
    }

    // 8. Persist.
    storage.save(library).map_err(ImportError::Storage)?;

    Ok(id)
}