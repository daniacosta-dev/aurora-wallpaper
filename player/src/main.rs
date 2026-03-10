mod dbus;
mod player;

use gtk::prelude::*;

fn main() {
    let app = gtk::Application::builder()
        .application_id("dev.daniacosta.AuroraWall.Player")
        .build();

    app.connect_activate(|app| {
        let state = player::PlayerState::new(app);
        dbus::start_dbus_service(state);
    });

    std::process::exit(app.run().into());
}