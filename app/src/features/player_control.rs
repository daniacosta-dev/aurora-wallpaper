use gtk::gio;
use gtk::gio::prelude::*;
use gtk::glib;

const PLAYER_BINARY: &str = "aurora-player";

/// Launch the player process if it's not already running, then send Play(path).
pub fn activate_wallpaper(path: &str) {
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

fn launch_player() {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()));

    let binary = exe_dir
        .map(|d| d.join(PLAYER_BINARY))
        .filter(|p| p.exists())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| PLAYER_BINARY.to_string());

    match std::process::Command::new(&binary).spawn() {
        Ok(_) => println!("[AuroraWall] Launched player: {binary}"),
        Err(e) => eprintln!("[AuroraWall] Failed to launch player ({binary}): {e}"),
    }
}