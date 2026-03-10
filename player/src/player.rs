use gdk_x11::prelude::*;
use gdk_x11::{X11Display, X11Surface};
use gtk::gdk;
use gtk::prelude::*;
use gtk::{self, glib};
use libmpv::Mpv;
use std::cell::RefCell;
use std::rc::Rc;

use crate::window_monitor::{X11WindowMonitor, WindowStateMonitor};

// ── Per-monitor player ────────────────────────────────────────────────────────

struct MonitorPlayer {
    window: gtk::ApplicationWindow,
    mpv: Option<Mpv>,
    connector: String,
    /// Monitor geometry for overlap detection.
    geo_x: i32,
    geo_y: i32,
    geo_w: i32,
    geo_h: i32,
    /// Whether playback is paused due to a covered window (not user-initiated).
    auto_paused: bool,
}

impl MonitorPlayer {
    fn new(app: &gtk::Application, monitor: &gdk::Monitor, index: usize) -> Self {
        let geo = monitor.geometry();
        let connector = monitor
            .connector()
            .unwrap_or_else(|| format!("monitor-{index}").into());

        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title(format!("AuroraWall Player [{connector}]"))
            .default_width(geo.width())
            .default_height(geo.height())
            .decorated(false)
            .build();

        let x = geo.x();
        let y = geo.y();
        let w = geo.width();
        let h = geo.height();
        let connector_clone = connector.clone();

        window.connect_realize(move |win| {
            set_desktop_window_hint(win, x, y, w, h, &connector_clone);
        });

        Self {
            window,
            mpv: None,
            connector: connector.to_string(),
            geo_x: geo.x(),
            geo_y: geo.y(),
            geo_w: geo.width(),
            geo_h: geo.height(),
            auto_paused: false,
        }
    }

    fn play(&mut self, path: &str) {
        if self.mpv.is_none() {
            let xid = get_xid(&self.window);
            match create_mpv(xid) {
                Ok(mpv) => self.mpv = Some(mpv),
                Err(e) => {
                    eprintln!("[aurora-player][{}] Failed to create mpv: {e}", self.connector);
                    return;
                }
            }
        }
        if let Some(mpv) = &self.mpv {
            if let Err(e) = mpv.command("loadfile", &[path, "replace"]) {
                eprintln!("[aurora-player][{}] loadfile error: {e}", self.connector);
            } else {
                self.auto_paused = false;
                println!("[aurora-player][{}] Playing: {path}", self.connector);
            }
        }
    }

    fn pause(&self) {
        if let Some(mpv) = &self.mpv {
            let _: Result<(), _> = mpv.set_property("pause", true);
        }
    }

    fn resume(&self) {
        if let Some(mpv) = &self.mpv {
            let _: Result<(), _> = mpv.set_property("pause", false);
        }
    }

    fn stop(&mut self) {
        if let Some(mpv) = &self.mpv {
            let _: Result<(), _> = mpv.command("stop", &[] as &[&str]);
        }
        self.auto_paused = false;
    }

    fn is_playing(&self) -> bool {
        self.mpv.is_some()
    }
}

// ── PlayerState ───────────────────────────────────────────────────────────────

pub struct PlayerState {
    pub app: gtk::Application,
    pub monitors: Vec<MonitorPlayer>,
    pub current_path: Option<String>,
}

impl PlayerState {
    pub fn new(app: &gtk::Application) -> Rc<RefCell<Self>> {
        let state = Rc::new(RefCell::new(PlayerState {
            app: app.clone(),
            monitors: Vec::new(),
            current_path: None,
        }));

        Self::rebuild_monitors(&state);
        Self::watch_monitors(&state);
        Self::start_auto_pause_loop(&state);

        state
    }

    fn rebuild_monitors(state: &Rc<RefCell<Self>>) {
        let display = gdk::Display::default().expect("No GDK display");
        let monitor_list = display.monitors();
        let n = monitor_list.n_items();

        println!("[aurora-player] Detected {n} monitor(s)");

        let mut st = state.borrow_mut();

        for mp in st.monitors.drain(..) {
            mp.window.close();
        }

        let app = st.app.clone();
        let current_path = st.current_path.clone();
        drop(st);

        let mut new_monitors: Vec<MonitorPlayer> = Vec::new();

        for i in 0..n {
            let monitor = monitor_list
                .item(i)
                .and_downcast::<gdk::Monitor>()
                .expect("Expected GdkMonitor");

            let mut mp = MonitorPlayer::new(&app, &monitor, i as usize);
            mp.window.present();

            if let Some(ref path) = current_path {
                mp.play(path);
            }

            new_monitors.push(mp);
        }

        state.borrow_mut().monitors = new_monitors;
    }

    fn watch_monitors(state: &Rc<RefCell<Self>>) {
        let state_clone = Rc::clone(state);
        let last_count = Rc::new(RefCell::new(
            gdk::Display::default()
                .map(|d| d.monitors().n_items())
                .unwrap_or(0),
        ));

        glib::timeout_add_local(std::time::Duration::from_secs(5), move || {
            let current = gdk::Display::default()
                .map(|d| d.monitors().n_items())
                .unwrap_or(0);

            let prev = *last_count.borrow();
            if current != prev {
                println!("[aurora-player] Monitor count changed {prev} → {current}, rebuilding");
                *last_count.borrow_mut() = current;
                Self::rebuild_monitors(&state_clone);
            }

            glib::ControlFlow::Continue
        });
    }

    /// Poll every second — pause monitors with a maximized window, resume the rest.
    fn start_auto_pause_loop(state: &Rc<RefCell<Self>>) {
        let state_clone = Rc::clone(state);

        // Open a dedicated X11 connection for window monitoring.
        let monitor = X11WindowMonitor::new(None);
        if monitor.is_none() {
            eprintln!("[aurora-player] Auto-pause disabled: could not open X11 connection");
            return;
        }
        let monitor = Rc::new(monitor.unwrap());

        glib::timeout_add_local(std::time::Duration::from_millis(1000), move || {
            let mut st = state_clone.borrow_mut();

            for mp in &mut st.monitors {
                if !mp.is_playing() {
                    continue;
                }

                let covered = monitor.has_covered_window(mp.geo_x, mp.geo_y, mp.geo_w, mp.geo_h);

                if covered && !mp.auto_paused {
                    mp.pause();
                    mp.auto_paused = true;
                    println!("[aurora-player][{}] Auto-paused (window covered)", mp.connector);
                } else if !covered && mp.auto_paused {
                    mp.resume();
                    mp.auto_paused = false;
                    println!("[aurora-player][{}] Auto-resumed", mp.connector);
                }
            }

            glib::ControlFlow::Continue
        });
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

pub fn play_all(state: &mut PlayerState, path: &str) {
    state.current_path = Some(path.to_string());
    for mp in &mut state.monitors {
        mp.play(path);
    }
}

pub fn play_on_monitor(state: &mut PlayerState, path: &str, index: usize) {
    if let Some(mp) = state.monitors.get_mut(index) {
        mp.play(path);
    } else {
        eprintln!("[aurora-player] No monitor at index {index}");
    }
}

pub fn pause_all(state: &mut PlayerState) {
    for mp in &mut state.monitors {
        mp.pause();
        mp.auto_paused = false; // user-initiated, reset auto flag
    }
}

pub fn resume_all(state: &mut PlayerState) {
    for mp in &mut state.monitors {
        mp.resume();
        mp.auto_paused = false;
    }
}

pub fn stop_all(state: &mut PlayerState) {
    for mp in &mut state.monitors {
        mp.stop();
    }
}

pub fn get_monitors(state: &PlayerState) -> Vec<String> {
    state
        .monitors
        .iter()
        .enumerate()
        .map(|(i, mp)| format!("{i}:{}", mp.connector))
        .collect()
}

// ── Helpers ───────────────────────────────────────────────────────────────────

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
    mpv.set_property("vo", "gpu")
        .map_err(|e| format!("mpv vo error: {e}"))?;
    mpv.set_property("hwdec", "auto-safe")
        .map_err(|e| format!("mpv hwdec error: {e}"))?;
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

fn set_desktop_window_hint(
    window: &gtk::ApplicationWindow,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    connector: &str,
) {
    let Some(surface) = window.surface() else { return };
    let Ok(x11_surface) = surface.downcast::<X11Surface>() else {
        eprintln!("[aurora-player][{connector}] Not on X11, skipping hint");
        return;
    };

    let xid = x11_surface.xid();
    let display = gtk::prelude::WidgetExt::display(window);
    let Ok(x11_display) = display.downcast::<X11Display>() else { return };

    let xdisplay = unsafe {
        gdk_x11::ffi::gdk_x11_display_get_xdisplay(
            x11_display.as_ptr() as *mut gdk_x11::ffi::GdkX11Display,
        )
    } as usize;

    let connector = connector.to_string();

    unsafe {
        apply_x11_desktop_hint(xdisplay as *mut std::ffi::c_void, xid, x, y, w, h);
    }

    glib::timeout_add_local_once(std::time::Duration::from_millis(300), move || {
        unsafe {
            apply_x11_desktop_hint(xdisplay as *mut std::ffi::c_void, xid, x, y, w, h);
        }
        println!("[aurora-player][{connector}] X11 hint re-applied xid={xid} geo={w}x{h}+{x}+{y}");
    });
}

unsafe fn apply_x11_desktop_hint(
    display: *mut std::ffi::c_void,
    xid: u64,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
) {
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
        fn XMoveResizeWindow(
            display: *mut std::ffi::c_void,
            window: u64,
            x: i32,
            y: i32,
            width: u32,
            height: u32,
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

    XMoveResizeWindow(display, xid, x, y, w as u32, h as u32);
    XLowerWindow(display, xid);
    XFlush(display);
}