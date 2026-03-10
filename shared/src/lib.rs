/// DBus interface constants shared between app and player.
/// Single source of truth — changing here updates both binaries.

pub const DBUS_NAME: &str = "dev.daniacosta.AuroraWall.Player";
pub const DBUS_PATH: &str = "/dev/daniacosta/AuroraWall/Player";
pub const DBUS_INTERFACE: &str = "dev.daniacosta.AuroraWall.Player";

use std::path::PathBuf;

/// Persists the currently active wallpaper path.
/// Storage: ~/.local/share/aurorawall/active.json
pub struct ActiveWallpaperStorage {
    path: PathBuf,
}

impl ActiveWallpaperStorage {
    pub fn new() -> Result<Self, ActiveStorageError> {
        let data_dir = dirs::data_local_dir()
            .ok_or(ActiveStorageError::NoDataDir)?
            .join("aurorawall");
        Ok(Self { path: data_dir.join("active.json") })
    }

    pub fn load(&self) -> Option<String> {
        let content = std::fs::read_to_string(&self.path).ok()?;
        serde_json::from_str::<String>(&content).ok()
    }

    pub fn save(&self, path: &str) -> Result<(), ActiveStorageError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(ActiveStorageError::Io)?;
        }
        let content = serde_json::to_string(path).map_err(ActiveStorageError::Json)?;
        let tmp = self.path.with_extension("json.tmp");
        std::fs::write(&tmp, &content).map_err(ActiveStorageError::Io)?;
        std::fs::rename(&tmp, &self.path).map_err(ActiveStorageError::Io)?;
        Ok(())
    }

    pub fn clear(&self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[derive(Debug)]
pub enum ActiveStorageError {
    NoDataDir,
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl std::fmt::Display for ActiveStorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActiveStorageError::NoDataDir => write!(f, "Could not determine XDG data directory"),
            ActiveStorageError::Io(e) => write!(f, "I/O error: {e}"),
            ActiveStorageError::Json(e) => write!(f, "JSON error: {e}"),
        }
    }
}

/// Manages the XDG autostart entry for aurora-player.
pub struct AutostartManager;

impl AutostartManager {
    fn desktop_path() -> Option<std::path::PathBuf> {
        dirs::config_dir().map(|d| d.join("autostart").join("aurora-player.desktop"))
    }

    pub fn is_enabled() -> bool {
        Self::desktop_path().map(|p| p.exists()).unwrap_or(false)
    }

    pub fn enable(player_binary_path: &str) -> Result<(), std::io::Error> {
        let path = Self::desktop_path().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "No XDG config dir")
        })?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name=AuroraWall Player\n\
             Comment=Video wallpaper player for AuroraWall\n\
             Exec={player_binary_path}\n\
             Hidden=false\n\
             NoDisplay=true\n\
             X-GNOME-Autostart-enabled=true\n"
        );

        std::fs::write(&path, content)?;
        println!("[AuroraWall] Autostart enabled: {}", path.display());
        Ok(())
    }

    pub fn disable() -> Result<(), std::io::Error> {
        if let Some(path) = Self::desktop_path() {
            if path.exists() {
                std::fs::remove_file(&path)?;
                println!("[AuroraWall] Autostart disabled");
            }
        }
        Ok(())
    }
}