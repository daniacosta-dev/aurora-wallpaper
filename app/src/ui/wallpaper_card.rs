use gtk::prelude::*;

use crate::domain::{Wallpaper, WallpaperId};

/// Build a wallpaper card row.
///
/// `on_remove` is called with the wallpaper's ID when the delete button is clicked.
/// The caller (window.rs) owns the state and handles the actual removal.
pub fn build_wallpaper_card<F, G>(
    wallpaper: &Wallpaper,
    on_activate: F,
    on_remove: G,
) -> gtk::ListBoxRow
where
    F: Fn(String) + 'static,
    G: Fn(WallpaperId) + 'static,
{
    let row = gtk::ListBoxRow::new();
    row.set_activatable(false);
    row.set_margin_top(4);
    row.set_margin_bottom(4);
    row.set_margin_start(6);
    row.set_margin_end(6);

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    hbox.set_margin_top(10);
    hbox.set_margin_bottom(10);
    hbox.set_margin_start(12);
    hbox.set_margin_end(12);

    // Thumbnail.
    let thumb_widget: gtk::Widget = if let Some(thumb_path) = &wallpaper.thumbnail_path {
        if thumb_path.exists() {
            let image = gtk::Image::from_file(thumb_path);
            image.set_size_request(120, 68);
            image.set_valign(gtk::Align::Center);
            image.upcast()
        } else {
            fallback_thumb().upcast()
        }
    } else {
        fallback_thumb().upcast()
    };

    // Metadata.
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 4);
    vbox.set_hexpand(true);
    vbox.set_valign(gtk::Align::Center);

    let name_label = gtk::Label::new(Some(wallpaper.display_name()));
    name_label.set_halign(gtk::Align::Start);
    name_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    name_label.add_css_class("heading");

    let path_str = wallpaper.file_path.to_string_lossy();
    let path_label = gtk::Label::new(Some(&path_str));
    path_label.set_halign(gtk::Align::Start);
    path_label.set_ellipsize(gtk::pango::EllipsizeMode::Start);
    path_label.add_css_class("caption");
    path_label.add_css_class("dim-label");

    vbox.append(&name_label);
    vbox.append(&path_label);

    if !wallpaper.source_exists() {
        let warn = gtk::Label::new(Some("⚠ File not found"));
        warn.add_css_class("caption");
        warn.set_halign(gtk::Align::Start);
        vbox.append(&warn);
    }
    // Set as Wallpaper button.
    let activate_btn = gtk::Button::with_label("Set as Wallpaper");
    activate_btn.add_css_class("suggested-action");
    activate_btn.add_css_class("pill");
    activate_btn.set_valign(gtk::Align::Center);

    let file_path = wallpaper.file_path.to_string_lossy().to_string();
    activate_btn.connect_clicked(move |_| {
        on_activate(file_path.clone());
    });

    // Delete button.
    let delete_btn = gtk::Button::new();
    delete_btn.set_icon_name("user-trash-symbolic");
    delete_btn.add_css_class("flat");
    delete_btn.add_css_class("circular");
    delete_btn.set_valign(gtk::Align::Center);
    delete_btn.set_tooltip_text(Some("Remove from library"));

    let wallpaper_id = wallpaper.id;
    delete_btn.connect_clicked(move |_| {
        on_remove(wallpaper_id);
    });

    hbox.append(&thumb_widget);
    hbox.append(&vbox);
    hbox.append(&activate_btn);
    hbox.append(&delete_btn);
    row.set_child(Some(&hbox));
    row
}

fn fallback_thumb() -> gtk::Box {
    let thumb = gtk::Box::new(gtk::Orientation::Vertical, 0);
    thumb.set_size_request(120, 68);
    thumb.add_css_class("card");
    thumb.set_valign(gtk::Align::Center);

    let icon = gtk::Image::from_icon_name("video-x-generic-symbolic");
    icon.set_pixel_size(32);
    icon.set_valign(gtk::Align::Center);
    icon.set_halign(gtk::Align::Center);
    icon.set_vexpand(true);
    thumb.append(&icon);
    thumb
}

pub fn build_empty_state() -> gtk::Box {
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 16);
    vbox.set_valign(gtk::Align::Center);
    vbox.set_halign(gtk::Align::Center);
    vbox.set_vexpand(true);

    let icon = gtk::Image::from_icon_name("video-x-generic-symbolic");
    icon.set_pixel_size(64);
    icon.add_css_class("dim-label");

    let label = gtk::Label::new(Some("No wallpapers yet"));
    label.add_css_class("title-2");

    let sublabel = gtk::Label::new(Some("Import a video to get started"));
    sublabel.add_css_class("dim-label");

    vbox.append(&icon);
    vbox.append(&label);
    vbox.append(&sublabel);
    vbox
}
