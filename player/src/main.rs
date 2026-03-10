mod dbus;
mod player;

use gtk::prelude::*;

fn main() {
    // Required by libmpv before initialization.
    unsafe {
        libc::setlocale(libc::LC_NUMERIC, c"C".as_ptr());
    }

    let app = gtk::Application::builder()
        .application_id("dev.daniacosta.AuroraWall.Player")
        .build();

    app.connect_activate(|app| {
        let window = player::build_player_window(app);
        window.present();
        dbus::start_dbus_service(window);
    });

    std::process::exit(app.run().into());
}