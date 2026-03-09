use std::path::PathBuf;

/// Unique identifier for a wallpaper entry.
pub type WallpaperId = u64;

/// Represents a video wallpaper managed by AuroraWall.
///
/// This is a pure domain type — no GTK, no I/O.
/// All fields use owned types so this can be freely cloned and serialized.
#[derive(Debug, Clone, PartialEq)]
pub struct Wallpaper {
    pub id: WallpaperId,
    pub name: String,
    pub file_path: PathBuf,
    pub thumbnail_path: Option<PathBuf>,
    pub duration_secs: Option<f64>,
    pub resolution: Option<(u32, u32)>,
    pub created_at: u64, // Unix timestamp
}

impl Wallpaper {
    /// Create a new wallpaper from a file path.
    /// `id` and `created_at` should be provided by the caller (e.g. infrastructure layer).
    pub fn new(id: WallpaperId, name: impl Into<String>, file_path: PathBuf, created_at: u64) -> Self {
        Self {
            id,
            name: name.into(),
            file_path,
            thumbnail_path: None,
            duration_secs: None,
            resolution: None,
            created_at,
        }
    }

    /// Derive a display name from the file stem if no name was given.
    pub fn display_name(&self) -> &str {
        &self.name
    }

    /// Returns true if the source file still exists on disk.
    pub fn source_exists(&self) -> bool {
        self.file_path.exists()
    }
}

/// In-memory collection of wallpapers with basic operations.
///
/// This is the application's core state — deliberately kept simple.
/// Persistence is handled separately in `infrastructure::storage`.
#[derive(Debug, Default, Clone)]
pub struct WallpaperLibrary {
    wallpapers: Vec<Wallpaper>,
    next_id: WallpaperId,
}

impl WallpaperLibrary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Restore library from a pre-loaded list (used by storage layer on startup).
    pub fn from_vec(wallpapers: Vec<Wallpaper>) -> Self {
        let next_id = wallpapers.iter().map(|w| w.id + 1).max().unwrap_or(1);
        Self { wallpapers, next_id }
    }

    /// Add a new wallpaper and return its assigned ID.
    pub fn add(&mut self, name: impl Into<String>, file_path: PathBuf, created_at: u64) -> WallpaperId {
        let id = self.next_id;
        self.next_id += 1;
        self.wallpapers.push(Wallpaper::new(id, name, file_path, created_at));
        id
    }

    pub fn remove(&mut self, id: WallpaperId) {
        self.wallpapers.retain(|w| w.id != id);
    }

    pub fn get(&self, id: WallpaperId) -> Option<&Wallpaper> {
        self.wallpapers.iter().find(|w| w.id == id)
    }

    pub fn get_mut(&mut self, id: WallpaperId) -> Option<&mut Wallpaper> {
        self.wallpapers.iter_mut().find(|w| w.id == id)
    }

    pub fn all(&self) -> &[Wallpaper] {
        &self.wallpapers
    }

    pub fn is_empty(&self) -> bool {
        self.wallpapers.is_empty()
    }

    pub fn len(&self) -> usize {
        self.wallpapers.len()
    }
}