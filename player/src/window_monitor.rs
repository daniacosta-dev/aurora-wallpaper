/// Window state monitoring abstraction.
/// Currently implemented for X11. Wayland implementation to be added later.

// ── Trait (abstraction for future Wayland support) ────────────────────────────

pub trait WindowStateMonitor {
    /// Returns true if there is a maximized or fullscreen window on the given monitor geometry.
    fn has_covered_window(&self, monitor_x: i32, monitor_y: i32, monitor_w: i32, monitor_h: i32) -> bool;
}

// ── X11 Implementation ────────────────────────────────────────────────────────

use x11::xlib;

pub struct X11WindowMonitor {
    display: *mut xlib::Display,
    root: xlib::Window,
    atoms: X11Atoms,
}

struct X11Atoms {
    net_client_list: xlib::Atom,
    net_wm_state: xlib::Atom,
    net_wm_state_maximized_vert: xlib::Atom,
    net_wm_state_maximized_horz: xlib::Atom,
    net_wm_state_fullscreen: xlib::Atom,
    net_wm_state_hidden: xlib::Atom,
    net_wm_desktop: xlib::Atom,
    net_current_desktop: xlib::Atom,
}

impl X11WindowMonitor {
    pub fn new(display_name: Option<&str>) -> Option<Self> {
        unsafe {
            let dpy = match display_name {
                Some(name) => {
                    let c = std::ffi::CString::new(name).ok()?;
                    xlib::XOpenDisplay(c.as_ptr())
                }
                None => xlib::XOpenDisplay(std::ptr::null()),
            };

            if dpy.is_null() {
                eprintln!("[aurora-player] X11WindowMonitor: could not open display");
                return None;
            }

            let root = xlib::XDefaultRootWindow(dpy);

            let atoms = X11Atoms {
                net_client_list: intern_atom(dpy, "_NET_CLIENT_LIST"),
                net_wm_state: intern_atom(dpy, "_NET_WM_STATE"),
                net_wm_state_maximized_vert: intern_atom(dpy, "_NET_WM_STATE_MAXIMIZED_VERT"),
                net_wm_state_maximized_horz: intern_atom(dpy, "_NET_WM_STATE_MAXIMIZED_HORZ"),
                net_wm_state_fullscreen: intern_atom(dpy, "_NET_WM_STATE_FULLSCREEN"),
                net_wm_state_hidden: intern_atom(dpy, "_NET_WM_STATE_HIDDEN"),
                net_wm_desktop: intern_atom(dpy, "_NET_WM_DESKTOP"),
                net_current_desktop: intern_atom(dpy, "_NET_CURRENT_DESKTOP"),
            };

            Some(Self { display: dpy, root, atoms })
        }
    }

    fn client_list(&self) -> Vec<xlib::Window> {
        unsafe {
            let mut actual_type: xlib::Atom = 0;
            let mut actual_format: i32 = 0;
            let mut n_items: u64 = 0;
            let mut bytes_after: u64 = 0;
            let mut data: *mut u8 = std::ptr::null_mut();

            let result = xlib::XGetWindowProperty(
                self.display,
                self.root,
                self.atoms.net_client_list,
                0,
                1024,
                xlib::False,
                xlib::XA_WINDOW,
                &mut actual_type,
                &mut actual_format,
                &mut n_items,
                &mut bytes_after,
                &mut data,
            );

            if result != xlib::Success as i32 || data.is_null() {
                return vec![];
            }

            let windows = std::slice::from_raw_parts(
                data as *const xlib::Window,
                n_items as usize,
            ).to_vec();
            xlib::XFree(data as *mut _);
            windows
        }
    }

    fn wm_state(&self, window: xlib::Window) -> Vec<xlib::Atom> {
        unsafe {
            let mut actual_type: xlib::Atom = 0;
            let mut actual_format: i32 = 0;
            let mut n_items: u64 = 0;
            let mut bytes_after: u64 = 0;
            let mut data: *mut u8 = std::ptr::null_mut();

            let result = xlib::XGetWindowProperty(
                self.display,
                window,
                self.atoms.net_wm_state,
                0,
                1024,
                xlib::False,
                xlib::XA_ATOM,
                &mut actual_type,
                &mut actual_format,
                &mut n_items,
                &mut bytes_after,
                &mut data,
            );

            if result != xlib::Success as i32 || data.is_null() {
                return vec![];
            }

            let atoms = std::slice::from_raw_parts(
                data as *const xlib::Atom,
                n_items as usize,
            ).to_vec();
            xlib::XFree(data as *mut _);
            atoms
        }
    }

    fn window_rect(&self, window: xlib::Window) -> Option<(i32, i32, i32, i32)> {
    unsafe {
        // Walk up to find the top-level frame window.
        let mut parent = window;
        let mut root_return: xlib::Window = 0;
        loop {
            let mut p: xlib::Window = 0;
            let mut children: *mut xlib::Window = std::ptr::null_mut();
            let mut n: u32 = 0;
            xlib::XQueryTree(self.display, parent, &mut root_return, &mut p, &mut children, &mut n);
            if !children.is_null() {
                xlib::XFree(children as *mut _);
            }
            if p == root_return || p == 0 {
                break;
            }
            parent = p;
        }

        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut w: u32 = 0;
        let mut h: u32 = 0;
        let mut border: u32 = 0;
        let mut depth: u32 = 0;
        let mut root_ret: xlib::Window = 0;

        let status = xlib::XGetGeometry(
            self.display, parent,
            &mut root_ret, &mut x, &mut y,
            &mut w, &mut h, &mut border, &mut depth,
        );

        if status == 0 { return None; }

        let mut child: xlib::Window = 0;
        xlib::XTranslateCoordinates(
            self.display, parent, self.root,
            0, 0, &mut x, &mut y, &mut child,
        );

        Some((x, y, w as i32, h as i32))
    }
}

    fn current_desktop(&self) -> u64 {
        unsafe {
            get_cardinal(self.display, self.root, self.atoms.net_current_desktop).unwrap_or(0)
        }
    }

    fn window_desktop(&self, window: xlib::Window) -> Option<u64> {
        unsafe {
            get_cardinal(self.display, window, self.atoms.net_wm_desktop)
        }
    }
}

impl WindowStateMonitor for X11WindowMonitor {
    fn has_covered_window(
        &self,
        monitor_x: i32,
        monitor_y: i32,
        monitor_w: i32,
        monitor_h: i32,
    ) -> bool {
        let current_desktop = self.current_desktop();

        for window in self.client_list() {
            let state = self.wm_state(window);
    
    let is_maximized = state.contains(&self.atoms.net_wm_state_maximized_vert)
        && state.contains(&self.atoms.net_wm_state_maximized_horz);
    let is_fullscreen = state.contains(&self.atoms.net_wm_state_fullscreen);
    
            // Skip windows on other desktops (0xFFFFFFFF = sticky/all desktops).
            if let Some(desktop) = self.window_desktop(window) {
                const ALL_DESKTOPS: u64 = 0xFFFFFFFF;
const ALL_DESKTOPS_U64: u64 = u64::MAX; // 18446744073709551615
if desktop != current_desktop && desktop != ALL_DESKTOPS && desktop != ALL_DESKTOPS_U64 {
    continue;
}
            }

            let state = self.wm_state(window);

            // Skip hidden (minimized) windows.
            if state.contains(&self.atoms.net_wm_state_hidden) {
                continue;
            }

            let is_maximized = state.contains(&self.atoms.net_wm_state_maximized_vert)
                && state.contains(&self.atoms.net_wm_state_maximized_horz);
            let is_fullscreen = state.contains(&self.atoms.net_wm_state_fullscreen);

            if !is_maximized && !is_fullscreen {
                continue;
            }

            if let Some((wx, wy, ww, wh)) = self.window_rect(window) {
    let ix = wx.max(monitor_x);
    let iy = wy.max(monitor_y);
    let ix2 = (wx + ww).min(monitor_x + monitor_w);
    let iy2 = (wy + wh).min(monitor_y + monitor_h);

    if ix2 > ix && iy2 > iy {
        let intersection = (ix2 - ix) * (iy2 - iy);
        let monitor_area = monitor_w * monitor_h;
        if intersection > monitor_area / 2 {
            return true;
        }
    }
}
        }

        false
    }
}

impl Drop for X11WindowMonitor {
    fn drop(&mut self) {
        unsafe {
            xlib::XCloseDisplay(self.display);
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

unsafe fn intern_atom(display: *mut xlib::Display, name: &str) -> xlib::Atom {
    let c = std::ffi::CString::new(name).unwrap();
    xlib::XInternAtom(display, c.as_ptr(), xlib::False)
}

unsafe fn get_cardinal(
    display: *mut xlib::Display,
    window: xlib::Window,
    atom: xlib::Atom,
) -> Option<u64> {
    let mut actual_type: xlib::Atom = 0;
    let mut actual_format: i32 = 0;
    let mut n_items: u64 = 0;
    let mut bytes_after: u64 = 0;
    let mut data: *mut u8 = std::ptr::null_mut();

    let result = xlib::XGetWindowProperty(
        display,
        window,
        atom,
        0,
        1,
        xlib::False,
        xlib::XA_CARDINAL,
        &mut actual_type,
        &mut actual_format,
        &mut n_items,
        &mut bytes_after,
        &mut data,
    );

    if result != xlib::Success as i32 || data.is_null() {
        return None;
    }

    let value = *(data as *const u64);
    xlib::XFree(data as *mut _);
    Some(value)
}