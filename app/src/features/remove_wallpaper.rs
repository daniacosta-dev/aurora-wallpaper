use std::path::PathBuf;

use crate::domain::{WallpaperId, WallpaperLibrary};
use crate::infrastructure::{LibraryStorage, StorageError};

#[derive(Debug)]
pub enum RemoveError {
    NotFound(WallpaperId),
    Storage(StorageError),
}

impl std::fmt::Display for RemoveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RemoveError::NotFound(id) => write!(f, "Wallpaper id={id} not found"),
            RemoveError::Storage(e) => write!(f, "Storage error: {e}"),
        }
    }
}

/// Remove a wallpaper from the library and persist.
///
/// Also deletes the thumbnail file from disk if it exists.
/// Never touches the original video file — only AuroraWall-owned files.
pub fn remove_wallpaper(
    id: WallpaperId,
    library: &mut WallpaperLibrary,
    storage: &LibraryStorage,
) -> Result<(), RemoveError> {
    // Grab thumbnail path before removing from library.
    let thumbnail_path: Option<PathBuf> = library
        .get(id)
        .and_then(|w| w.thumbnail_path.clone());

    if library.get(id).is_none() {
        return Err(RemoveError::NotFound(id));
    }

    library.remove(id);
    storage.save(library).map_err(RemoveError::Storage)?;

    // Clean up thumbnail — best effort, non-fatal.
    if let Some(path) = thumbnail_path {
        if path.exists() {
            if let Err(e) = std::fs::remove_file(&path) {
                eprintln!("[AuroraWall] Could not delete thumbnail: {e}");
            }
        }
    }

    Ok(())
}