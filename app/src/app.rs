use adw::prelude::*;

use crate::ui::window::build_window;

/// Application ID — must match the Flatpak manifest in the future.
pub const APP_ID: &str = "io.github.daniacosta_dev.AuroraWall";

/// Build the `adw::Application` and wire up the `activate` signal.
pub fn build_app() -> adw::Application {
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(|app| {
        let window = build_window(app);
        window.present();
    });

    app
}