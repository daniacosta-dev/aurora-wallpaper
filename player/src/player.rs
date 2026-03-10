use gdk_x11::prelude::*;
use gdk_x11::{X11Display, X11Surface};
use gtk::prelude::*;
use gtk::{self, glib};
use libmpv::Mpv;
use std::cell::RefCell;
use std::rc::Rc;
use libc;

pub struct PlayerState {
    pub window: gtk::ApplicationWindow,
    pub mpv: Rc<RefCell<Option<Mpv>>>,
}

pub fn build_player_window(app: &gtk::Application) -> gtk::ApplicationWindow {
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("AuroraWall Player")
        .default_width(1920)
        .default_height(1080)
        .decorated(false)
        .build();

    window.connect_realize(|win| {
        set_desktop_window_hint(win);
    });

    window
}

pub fn play(state: &PlayerState, path: &str) {
    let xid = get_xid(&state.window);
    let mut mpv_ref = state.mpv.borrow_mut();

    if mpv_ref.is_none() {
        match create_mpv(xid) {
            Ok(mpv) => *mpv_ref = Some(mpv),
            Err(e) => { eprintln!("[aurora-player] Failed to create mpv: {e}"); return; }
        }
    }

    if let Some(mpv) = mpv_ref.as_ref() {
        if let Err(e) = mpv.command("loadfile", &[path, "replace"]) {
            eprintln!("[aurora-player] mpv loadfile error: {e}");
        } else {
            println!("[aurora-player] Playing: {path}");
        }
    }
}

pub fn pause(state: &PlayerState) {
    if let Some(mpv) = state.mpv.borrow().as_ref() {
        let _: Result<(), _> = mpv.set_property("pause", true);
        println!("[aurora-player] Paused");
    }
}

pub fn stop(state: &PlayerState) {
    if let Some(mpv) = state.mpv.borrow().as_ref() {
        let _: Result<(), _> = mpv.command("stop", &[] as &[&str]);
        println!("[aurora-player] Stopped");
    }
}

fn create_mpv(wid: u64) -> Result<Mpv, String> {

    unsafe {
        let c = std::ffi::CString::new("C").unwrap();
        libc::setlocale(libc::LC_NUMERIC, c.as_ptr());
    }
    
    let mpv = Mpv::new().map_err(|e| format!("mpv init error: {e}"))?;

    mpv.set_property("wid", wid as i64)
        .map_err(|e| format!("mpv wid error: {e}"))?;
    mpv.set_property("loop-file", "inf")
        .map_err(|e| format!("mpv loop error: {e}"))?;
    mpv.set_property("vo", "x11")
        .map_err(|e| format!("mpv vo error: {e}"))?;
    mpv.set_property("osc", false)
        .map_err(|e| format!("mpv osc error: {e}"))?;
    mpv.set_property("input-default-bindings", false)
        .map_err(|e| format!("mpv bindings error: {e}"))?;
    mpv.set_property("mute", true)
        .map_err(|e| format!("mpv mute error: {e}"))?;

    Ok(mpv)
}

fn get_xid(window: &gtk::ApplicationWindow) -> u64 {
    if let Some(surface) = window.surface() {
        if let Ok(x11_surface) = surface.downcast::<X11Surface>() {
            return x11_surface.xid();
        }
    }
    0
}

fn set_desktop_window_hint(window: &gtk::ApplicationWindow) {
    let Some(surface) = window.surface() else { return };
    let Ok(x11_surface) = surface.downcast::<X11Surface>() else {
        eprintln!("[aurora-player] Not on X11, skipping hint");
        return;
    };

    let xid = x11_surface.xid();
    let display = gtk::prelude::WidgetExt::display(window);
    let Ok(x11_display) = display.downcast::<X11Display>() else { return };

    let xdisplay = unsafe {
        gdk_x11::ffi::gdk_x11_display_get_xdisplay(
            x11_display.as_ptr() as *mut gdk_x11::ffi::GdkX11Display,
        )
    };

    unsafe {
        apply_x11_desktop_hint(xdisplay as *mut std::ffi::c_void, xid);
    }
}

unsafe fn apply_x11_desktop_hint(display: *mut std::ffi::c_void, xid: u64) {
    use std::ffi::CString;

    extern "C" {
        fn XInternAtom(
            display: *mut std::ffi::c_void,
            name: *const std::os::raw::c_char,
            only_if_exists: i32,
        ) -> u64;
        fn XChangeProperty(
            display: *mut std::ffi::c_void,
            window: u64,
            property: u64,
            type_: u64,
            format: i32,
            mode: i32,
            data: *const u8,
            nelements: i32,
        ) -> i32;
        fn XLowerWindow(display: *mut std::ffi::c_void, window: u64) -> i32;
        fn XFlush(display: *mut std::ffi::c_void) -> i32;
    }

    let wm_type_atom = XInternAtom(
        display,
        CString::new("_NET_WM_WINDOW_TYPE").unwrap().as_ptr(),
        0,
    );
    let desktop_atom = XInternAtom(
        display,
        CString::new("_NET_WM_WINDOW_TYPE_DESKTOP").unwrap().as_ptr(),
        0,
    );
    let atom_type = XInternAtom(display, CString::new("ATOM").unwrap().as_ptr(), 0);

    XChangeProperty(
        display,
        xid,
        wm_type_atom,
        atom_type,
        32,
        0,
        &desktop_atom as *const u64 as *const u8,
        1,
    );
    XLowerWindow(display, xid);
    XFlush(display);

    println!("[aurora-player] X11 desktop hint applied xid={xid}");
}