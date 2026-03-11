use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use gtk::prelude::*;

use crate::domain::WallpaperLibrary;
use crate::features::import_video::{import_video, SUPPORTED_EXTENSIONS};
use crate::features::remove_wallpaper::remove_wallpaper;
use crate::infrastructure::LibraryStorage;
use crate::ui::header::build_header_bar;
use crate::ui::wallpaper_card::{build_empty_state, build_wallpaper_card};

struct AppState {
    library: WallpaperLibrary,
    storage: LibraryStorage,
}

pub fn build_window(app: &adw::Application) -> adw::ApplicationWindow {
    // --- State ---
    let storage = LibraryStorage::new().expect("Could not initialise storage");
    let library = storage.load().unwrap_or_default();
    let state = Rc::new(RefCell::new(AppState { library, storage }));

    // --- Window ---
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("AuroraWall")
        .default_width(900)
        .default_height(650)
        .build();

    // --- Scrollable list ---
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_vexpand(true);
    scroll.set_hexpand(true);
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

    let list_box = gtk::ListBox::new();
    list_box.add_css_class("boxed-list");
    list_box.set_selection_mode(gtk::SelectionMode::None);
    list_box.set_margin_top(12);
    list_box.set_margin_bottom(12);
    list_box.set_margin_start(16);
    list_box.set_margin_end(16);

    scroll.set_child(Some(&list_box));

    // --- Stack: empty state vs list ---
    let stack = gtk::Stack::new();
    stack.set_transition_type(gtk::StackTransitionType::Crossfade);
    stack.add_named(&build_empty_state(), Some("empty"));
    stack.add_named(&scroll, Some("list"));

    // --- Action bar ---
    let action_bar = gtk::ActionBar::new();

    let stop_btn = gtk::Button::with_label("Stop Wallpaper");
    stop_btn.add_css_class("destructive-action");
    stop_btn.add_css_class("pill");
    action_bar.pack_start(&stop_btn);

    let import_btn = gtk::Button::with_label("Import Video");
    import_btn.add_css_class("suggested-action");
    import_btn.add_css_class("pill");
    action_bar.pack_end(&import_btn);

    // --- Content area ---
    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.append(&stack);
    content.append(&action_bar);

    // --- Main layout ---
    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let (header, settings_btn) = build_header_bar();
    main_box.append(&header);
    main_box.append(&content);

    window.set_content(Some(&main_box));

    // --- Populate on startup ---
    refresh_list(&list_box, &stack, &state);

    // --- Stop button ---
    stop_btn.connect_clicked(move |_| {
        crate::features::player_control::stop_wallpaper();
    });

    // --- Settings button ---
    let window_weak = window.downgrade();
    settings_btn.connect_clicked(move |_| {
        let Some(win) = window_weak.upgrade() else {
            return;
        };
        show_settings_dialog(&win);
    });

    // --- Import button ---
    let window_weak2 = window.downgrade();
    let state_clone = Rc::clone(&state);
    let list_clone = list_box.clone();
    let stack_clone = stack.clone();

    import_btn.connect_clicked(move |_| {
        let Some(win) = window_weak2.upgrade() else {
            return;
        };

        let filter = gtk::FileFilter::new();
        filter.set_name(Some("Video files"));
        for ext in SUPPORTED_EXTENSIONS {
            filter.add_pattern(&format!("*.{ext}"));
        }

        let dialog = gtk::FileChooserDialog::new(
            Some("Import Video"),
            Some(&win),
            gtk::FileChooserAction::Open,
            &[
                ("Cancel", gtk::ResponseType::Cancel),
                ("Open", gtk::ResponseType::Accept),
            ],
        );
        dialog.add_filter(&filter);

        let state_cb = Rc::clone(&state_clone);
        let list_cb = list_clone.clone();
        let stack_cb = stack_clone.clone();
        let win_for_error = win.clone();

        dialog.connect_response(move |d, response| {
            if response == gtk::ResponseType::Accept {
                if let Some(file) = d.file() {
                    if let Some(path) = file.path() {
                        let storage = state_cb.borrow().storage.clone_path();
                        let result = {
                            let mut st = state_cb.borrow_mut();
                            import_video(path, &mut st.library, &storage)
                        };
                        match result {
                            Ok(id) => {
                                println!("[AuroraWall] Imported wallpaper id={id}");
                                refresh_list(&list_cb, &stack_cb, &state_cb);
                            }
                            Err(e) => {
                                eprintln!("[AuroraWall] Import error: {e}");
                                let msg = gtk::MessageDialog::new(
                                    Some(&win_for_error),
                                    gtk::DialogFlags::MODAL,
                                    gtk::MessageType::Error,
                                    gtk::ButtonsType::Ok,
                                    &e.to_string(),
                                );
                                msg.set_title(Some("Could not import video"));
                                msg.connect_response(|d, _| d.close());
                                msg.show();
                            }
                        }
                    }
                }
            }
            d.close();
        });

        dialog.show();
    });

    window
}

/// Shows the settings dialog.
fn show_settings_dialog(parent: &adw::ApplicationWindow) {
    let config_storage = match aurora_shared::AppConfigStorage::new() {
        Some(s) => s,
        None => return,
    };
    let config = config_storage.load();

    let dialog = gtk::Dialog::builder()
        .title("Settings")
        .transient_for(parent)
        .modal(true)
        .default_width(400)
        .build();

    dialog.add_button("Close", gtk::ResponseType::Close);
    dialog.add_button("Apply", gtk::ResponseType::Apply);

    let content = dialog.content_area();
    content.set_spacing(0);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(16);
    content.set_margin_end(16);

    // --- Performance section ---
    let section_label = gtk::Label::new(Some("Performance"));
    section_label.add_css_class("heading");
    section_label.set_halign(gtk::Align::Start);
    section_label.set_margin_bottom(8);
    content.append(&section_label);

    let perf_list = gtk::ListBox::new();
    perf_list.add_css_class("boxed-list");
    perf_list.set_selection_mode(gtk::SelectionMode::None);
    perf_list.set_margin_bottom(16);

    // Default mode row.
    let default_row = adw::ActionRow::new();
    default_row.set_title("Default");
    default_row.set_subtitle("Uses integrated GPU — lower power consumption");
    let default_radio = gtk::CheckButton::new();
    default_radio.set_valign(gtk::Align::Center);
    default_row.add_suffix(&default_radio);
    default_row.set_activatable_widget(Some(&default_radio));

    // High performance mode row.
    let perf_row = adw::ActionRow::new();
    perf_row.set_title("High Performance");
    perf_row.set_subtitle("Uses dedicated GPU (NVIDIA) — better decoding, higher power use");
    let perf_radio = gtk::CheckButton::new();
    perf_radio.set_valign(gtk::Align::Center);
    perf_radio.set_group(Some(&default_radio));
    perf_row.add_suffix(&perf_radio);
    perf_row.set_activatable_widget(Some(&perf_radio));

    // Disable High Performance option if no NVIDIA GPU is available.
    if !std::path::Path::new("/dev/nvidia0").exists() {
        perf_radio.set_sensitive(false);
        perf_row.set_subtitle("Uses dedicated GPU (NVIDIA) — not available on this system");
    }

    // Set initial state.
    if config.high_performance {
        perf_radio.set_active(true);
    } else {
        default_radio.set_active(true);
    }

    perf_list.append(&default_row);
    perf_list.append(&perf_row);
    content.append(&perf_list);

    // --- Note ---
    let note = gtk::Label::new(Some(
        "Changes take effect the next time the wallpaper is activated.",
    ));
    note.add_css_class("dim-label");
    note.add_css_class("caption");
    note.set_halign(gtk::Align::Start);
    note.set_wrap(true);
    content.append(&note);

    // --- Save on toggle ---
    perf_radio.connect_toggled(move |btn| {
        let high_performance = btn.is_active();
        let mut cfg = config_storage.load();
        cfg.high_performance = high_performance;
        if let Err(e) = config_storage.save(&cfg) {
            eprintln!("[AuroraWall] Could not save config: {e}");
        } else {
            println!(
                "[AuroraWall] Performance mode: {}",
                if high_performance { "high" } else { "default" }
            );
        }
    });

    dialog.connect_response(|d, response| {
    if response == gtk::ResponseType::Apply {
        crate::features::player_control::restart_player();
    }
    d.close();
    });
    dialog.show();
}

/// Rebuild the list from current state.
fn refresh_list(list_box: &gtk::ListBox, stack: &gtk::Stack, state: &Rc<RefCell<AppState>>) {
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }

    let is_empty = state.borrow().library.is_empty();
    if is_empty {
        stack.set_visible_child_name("empty");
        return;
    }

    let wallpapers: Vec<_> = state.borrow().library.all().to_vec();

    for wallpaper in &wallpapers {
        let state_for_card = Rc::clone(state);
        let list_for_card = list_box.clone();
        let stack_for_card = stack.clone();

        let card = build_wallpaper_card(
            wallpaper,
            |path| {
                crate::features::player_control::activate_wallpaper(&path);
            },
            move |id| {
                let name = state_for_card
                    .borrow()
                    .library
                    .get(id)
                    .map(|w| w.name.clone())
                    .unwrap_or_else(|| "this wallpaper".to_string());

                let confirm = gtk::MessageDialog::new(
                    None::<&gtk::Window>,
                    gtk::DialogFlags::MODAL,
                    gtk::MessageType::Question,
                    gtk::ButtonsType::None,
                    &format!("Remove \"{}\"?", name),
                );
                confirm.add_button("Cancel", gtk::ResponseType::Cancel);
                let delete_btn = confirm.add_button("Remove", gtk::ResponseType::Accept);
                delete_btn.add_css_class("destructive-action");
                confirm.set_secondary_text(Some(
                    "It will be removed from the library. The video file will not be deleted.",
                ));

                let state_cb = Rc::clone(&state_for_card);
                let list_cb = list_for_card.clone();
                let stack_cb = stack_for_card.clone();

                confirm.connect_response(move |d, response| {
                    d.close();
                    if response == gtk::ResponseType::Accept {
                        let storage = state_cb.borrow().storage.clone_path();
                        let result = {
                            let mut st = state_cb.borrow_mut();
                            remove_wallpaper(id, &mut st.library, &storage)
                        };
                        match result {
                            Ok(()) => {
                                println!("[AuroraWall] Removed wallpaper id={id}");
                                refresh_list(&list_cb, &stack_cb, &state_cb);
                            }
                            Err(e) => eprintln!("[AuroraWall] Remove error: {e}"),
                        }
                    }
                });

                confirm.show();
            },
        );

        list_box.append(&card);
    }

    stack.set_visible_child_name("list");
}