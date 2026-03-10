mod dbus;
mod player;
mod window_monitor;

use gtk::prelude::*;

fn main() {
    unsafe {
        let c = std::ffi::CString::new("C").unwrap();
        libc::setlocale(libc::LC_NUMERIC, c.as_ptr());
    }

    let app = gtk::Application::builder()
        .application_id("dev.daniacosta.AuroraWall.Player")
        .build();

    app.connect_activate(|app| {
        let state = player::PlayerState::new(app);

        let maybe_path = aurora_shared::ActiveWallpaperStorage::new()
            .ok()
            .and_then(|s| s.load());

        if let Some(path) = maybe_path {
            println!("[aurora-player] Resuming last wallpaper: {path}");
            let state_clone = std::rc::Rc::clone(&state);
            gtk::glib::timeout_add_local_once(
                std::time::Duration::from_millis(500),
                move || {
                    player::play_all(&mut state_clone.borrow_mut(), &path);
                },
            );
        }

        dbus::start_dbus_service(state);
    });

    std::process::exit(app.run().into());
}