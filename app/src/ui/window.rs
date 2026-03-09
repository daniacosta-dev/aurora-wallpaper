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
    let header = build_header_bar();
    main_box.append(&header);
    main_box.append(&content);

    window.set_content(Some(&main_box));

    // --- Populate on startup ---
    refresh_list(&list_box, &stack, &state);

    // --- Import button ---
    let window_weak = window.downgrade();
    let state_clone = Rc::clone(&state);
    let list_clone = list_box.clone();
    let stack_clone = stack.clone();

    import_btn.connect_clicked(move |_| {
        let Some(win) = window_weak.upgrade() else { return };

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

/// Rebuild the list from current state.
///
/// Takes the full `Rc<RefCell<AppState>>` so it can also wire up
/// the remove callback for each new card.
fn refresh_list(
    list_box: &gtk::ListBox,
    stack: &gtk::Stack,
    state: &Rc<RefCell<AppState>>,
) {
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }

    let is_empty = state.borrow().library.is_empty();
    if is_empty {
        stack.set_visible_child_name("empty");
        return;
    }

    // Collect IDs first to avoid holding the borrow while building cards.
    let wallpapers: Vec<_> = state.borrow().library.all().to_vec();

    for wallpaper in &wallpapers {
        let state_for_card = Rc::clone(state);
        let list_for_card = list_box.clone();
        let stack_for_card = stack.clone();

        let card = build_wallpaper_card(wallpaper, move |id| {
            let storage = state_for_card.borrow().storage.clone_path();
            let result = {
                let mut st = state_for_card.borrow_mut();
                remove_wallpaper(id, &mut st.library, &storage)
            };
            match result {
                Ok(()) => {
                    println!("[AuroraWall] Removed wallpaper id={id}");
                    refresh_list(&list_for_card, &stack_for_card, &state_for_card);
                }
                Err(e) => eprintln!("[AuroraWall] Remove error: {e}"),
            }
        });

        list_box.append(&card);
    }

    stack.set_visible_child_name("list");
}