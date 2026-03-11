use adw::prelude::*;
use gtk::prelude::*;

/// Builds the application header bar.
/// Returns (HeaderBar, settings_button) — the caller wires up signals.
pub fn build_header_bar() -> (adw::HeaderBar, gtk::Button) {
    let header = adw::HeaderBar::new();

    let title = adw::WindowTitle::new(
        "Aurora Video Wallpaper",
        "Video Wallpaper Manager for Linux",
    );
    header.set_title_widget(Some(&title));

    // Settings button.
    let settings_btn = gtk::Button::from_icon_name("preferences-system-symbolic");
    settings_btn.set_tooltip_text(Some("Settings"));
    header.pack_end(&settings_btn);

    (header, settings_btn)
}