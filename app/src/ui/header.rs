use adw::prelude::*;
use gtk::prelude::*;

/// Builds the application header bar.
///
/// Returns the `adw::HeaderBar` widget — the caller places it in the window.
/// Callback-based design: the caller wires up signals, keeping this widget dumb.
pub fn build_header_bar() -> adw::HeaderBar {
    let header = adw::HeaderBar::new();

    // Title widget — using AdwWindowTitle for the two-line style.
    let title = adw::WindowTitle::new("Aurora Wallpaper", "Video Wallpaper Manager for Linux");
    header.set_title_widget(Some(&title));

    header
}