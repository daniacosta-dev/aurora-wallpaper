use std::path::{Path, PathBuf};
use std::fs;

use crate::domain::{Wallpaper, WallpaperId, WallpaperLibrary};

/// Serializable DTO — decouples the domain model from the JSON format.
#[derive(serde::Serialize, serde::Deserialize)]
struct WallpaperRecord {
    id: WallpaperId,
    name: String,
    file_path: String,
    thumbnail_path: Option<String>,
    duration_secs: Option<f64>,
    resolution: Option<(u32, u32)>,
    created_at: u64,
}

impl From<&Wallpaper> for WallpaperRecord {
    fn from(w: &Wallpaper) -> Self {
        Self {
            id: w.id,
            name: w.name.clone(),
            file_path: w.file_path.to_string_lossy().into_owned(),
            thumbnail_path: w.thumbnail_path.as_ref().map(|p| p.to_string_lossy().into_owned()),
            duration_secs: w.duration_secs,
            resolution: w.resolution,
            created_at: w.created_at,
        }
    }
}

impl From<WallpaperRecord> for Wallpaper {
    fn from(r: WallpaperRecord) -> Self {
        Wallpaper {
            id: r.id,
            name: r.name,
            file_path: PathBuf::from(r.file_path),
            thumbnail_path: r.thumbnail_path.map(PathBuf::from),
            duration_secs: r.duration_secs,
            resolution: r.resolution,
            created_at: r.created_at,
        }
    }
}

/// Handles reading and writing the wallpaper library to disk.
/// Storage: ~/.local/share/aurorawall/library.json
pub struct LibraryStorage {
    path: PathBuf,
}

impl LibraryStorage {
    pub fn new() -> Result<Self, StorageError> {
        let data_dir = dirs::data_local_dir()
            .ok_or(StorageError::NoDataDir)?
            .join("aurorawall");
        Ok(Self { path: data_dir.join("library.json") })
    }

    pub fn with_path(path: impl AsRef<Path>) -> Self {
        Self { path: path.as_ref().to_owned() }
    }

    pub fn load(&self) -> Result<WallpaperLibrary, StorageError> {
        if !self.path.exists() {
            return Ok(WallpaperLibrary::new());
        }
        let content = fs::read_to_string(&self.path).map_err(StorageError::Io)?;
        let records: Vec<WallpaperRecord> = serde_json::from_str(&content).map_err(StorageError::Json)?;
        let wallpapers: Vec<Wallpaper> = records.into_iter().map(Into::into).collect();
        Ok(WallpaperLibrary::from_vec(wallpapers))
    }

    /// Atomic write: write to .tmp then rename.
    pub fn save(&self, library: &WallpaperLibrary) -> Result<(), StorageError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(StorageError::Io)?;
        }
        let records: Vec<WallpaperRecord> = library.all().iter().map(Into::into).collect();
        let content = serde_json::to_string_pretty(&records).map_err(StorageError::Json)?;

        let tmp_path = self.path.with_extension("json.tmp");
        fs::write(&tmp_path, &content).map_err(StorageError::Io)?;
        fs::rename(&tmp_path, &self.path).map_err(StorageError::Io)?;

        Ok(())
    }

    pub fn clone_path(&self) -> LibraryStorage {
        LibraryStorage { path: self.path.clone() }
    }
}

#[derive(Debug)]
pub enum StorageError {
    NoDataDir,
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::NoDataDir => write!(f, "Could not determine XDG data directory"),
            StorageError::Io(e) => write!(f, "I/O error: {e}"),
            StorageError::Json(e) => write!(f, "JSON error: {e}"),
        }
    }
}