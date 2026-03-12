use gtk::gio;
use gtk::gio::prelude::*;
use gtk::glib;

const PLAYER_BINARY: &str = "aurora-player";

/// Returns the correct path to aurora-player depending on the install method.
fn resolve_player_binary() -> String {
    // Running as a snap — use the public snap command name.
    if std::env::var("SNAP").is_ok() {
        return "aurora-video-wallpaper.aurora-player".to_string();
    }

    // Otherwise, look next to the current executable.
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join(PLAYER_BINARY)))
        .filter(|p| p.exists())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| PLAYER_BINARY.to_string())
}

/// Returns the correct autostart Exec path for aurora-player.
fn resolve_autostart_path() -> String {
    if std::env::var("SNAP").is_ok() {
        return "/snap/bin/aurora-video-wallpaper.aurora-player".to_string();
    }

    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join(PLAYER_BINARY)))
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| PLAYER_BINARY.to_string())
}

/// Launch the player process if it's not already running, then send Play(path).
pub fn activate_wallpaper(path: &str) {
    // Enable autostart on first activation.
    if !aurora_shared::AutostartManager::is_enabled() {
        let player_path = resolve_autostart_path();
        if let Err(e) = aurora_shared::AutostartManager::enable(&player_path) {
            eprintln!("[AuroraWall] Could not enable autostart: {e}");
        }
    }

    // Persist active wallpaper so the player can resume on next launch.
    if let Ok(storage) = aurora_shared::ActiveWallpaperStorage::new() {
        if let Err(e) = storage.save(&path) {
            eprintln!("[AuroraWall] Could not persist active wallpaper: {e}");
        }
    }

    let path = path.to_string();
    glib::MainContext::default().spawn_local(async move {
        // Try to call Play directly — if it fails, the player isn't running yet.
        if send_play(&path).await.is_err() {
            // Launch the player binary.
            launch_player();

            // Wait a moment for it to register on DBus.
            glib::timeout_future(std::time::Duration::from_millis(800)).await;

            // Try again.
            if let Err(e) = send_play(&path).await {
                eprintln!("[AuroraWall] Could not contact player after launch: {e}");
            }
        }
    });
}

async fn send_play(path: &str) -> Result<(), glib::Error> {
    let connection = gio::bus_get_future(gio::BusType::Session).await?;

    connection
        .call_future(
            Some(aurora_shared::DBUS_NAME),
            aurora_shared::DBUS_PATH,
            aurora_shared::DBUS_INTERFACE,
            "Play",
            Some(&glib::Variant::tuple_from_iter([glib::Variant::from(
                path,
            )])),
            None,
            gio::DBusCallFlags::NONE,
            3000,
        )
        .await
        .map(|_| {
            println!("[AuroraWall] Sent Play: {path}");
        })
}

/// Returns true if a discrete NVIDIA GPU is available for offloading.
fn is_nvidia_available() -> bool {
    std::path::Path::new("/dev/nvidia0").exists()
}

fn launch_player() {
    let binary = resolve_player_binary();

    // Read user config to determine performance mode.
    let high_performance = aurora_shared::AppConfigStorage::new()
        .map(|s| s.load().high_performance)
        .unwrap_or(false);

    let result = if high_performance && is_nvidia_available() {
        println!("[AuroraWall] High Performance mode — launching player with NVIDIA GPU offload");
        std::process::Command::new(&binary)
            .env("__NV_PRIME_RENDER_OFFLOAD", "1")
            .env("__GLX_VENDOR_LIBRARY_NAME", "nvidia")
            .env("__VK_LAYER_NV_optimus", "NVIDIA_only")
            .spawn()
    } else {
        if high_performance && !is_nvidia_available() {
            println!("[AuroraWall] High Performance requested but no NVIDIA GPU found, using default");
        }
        std::process::Command::new(&binary).spawn()
    };

    match result {
        Ok(_) => println!("[AuroraWall] Launched player: {binary}"),
        Err(e) => eprintln!("[AuroraWall] Failed to launch player ({binary}): {e}"),
    }
}

pub fn stop_wallpaper() {
    // Send DBus Stop first so the player can clean up active.json.
    let conn = match gio::bus_get_sync(gio::BusType::Session, None::<&gio::Cancellable>) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[AuroraWall] DBus connection error: {e}");
            return;
        }
    };

    let _ = conn.call_sync(
        Some(aurora_shared::DBUS_NAME),
        aurora_shared::DBUS_PATH,
        aurora_shared::DBUS_INTERFACE,
        "Stop",
        None,
        None,
        gio::DBusCallFlags::NONE,
        2000,
        None::<&gio::Cancellable>,
    );

    // Give the player a moment to clean up then kill it.
    std::thread::sleep(std::time::Duration::from_millis(200));
    std::process::Command::new("pkill")
        .arg("-f")
        .arg("aurora-player")
        .spawn()
        .ok();
}

pub fn restart_player() {
    // Kill existing player.
    std::process::Command::new("pkill")
        .arg("-f")
        .arg("aurora-player")
        .spawn()
        .ok();

    // Wait for it to die.
    std::thread::sleep(std::time::Duration::from_millis(300));

    // Relaunch with current config.
    launch_player();

    // Resume last wallpaper via DBus after player registers.
    glib::MainContext::default().spawn_local(async move {
        glib::timeout_future(std::time::Duration::from_millis(800)).await;

        if let Some(storage) = aurora_shared::ActiveWallpaperStorage::new().ok() {
            if let Some(path) = storage.load() {
                if let Err(e) = send_play(&path).await {
                    eprintln!("[AuroraWall] Could not resume after restart: {e}");
                }
            }
        }
    });
}