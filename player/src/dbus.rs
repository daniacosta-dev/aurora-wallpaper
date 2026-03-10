use glib;
use gtk::gio;
use gtk::gio::prelude::*;

use crate::player::{self, PlayerState};
use std::cell::RefCell;
use std::rc::Rc;

const DBUS_INTERFACE_XML: &str = r#"
<node>
  <interface name="dev.daniacosta.AuroraWall.Player">
    <method name="Play">
      <arg type="s" direction="in" name="path"/>
    </method>
    <method name="Pause"/>
    <method name="Stop"/>
  </interface>
</node>
"#;

pub fn start_dbus_service(window: gtk::ApplicationWindow) {
    let state = Rc::new(RefCell::new(PlayerState {
        window,
        mpv: Rc::new(RefCell::new(None)),
    }));

    glib::MainContext::default().spawn_local(async move {
        register(state).await;
    });
}

async fn register(state: Rc<RefCell<PlayerState>>) {
    let connection = match gio::bus_get_future(gio::BusType::Session).await {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("[aurora-player] DBus connection failed: {e}");
            return;
        }
    };

    let node_info = match gio::DBusNodeInfo::for_xml(DBUS_INTERFACE_XML) {
        Ok(info) => info,
        Err(e) => {
            eprintln!("[aurora-player] DBus interface parse error: {e}");
            return;
        }
    };

    let interface_info = node_info
        .lookup_interface("dev.daniacosta.AuroraWall.Player")
        .expect("Interface not found in XML");

    let state_for_methods = Rc::clone(&state);

    let reg_id = connection.register_object(
        "/dev/daniacosta/AuroraWall/Player",
        &interface_info,
        move |_conn, _sender, _path, _interface, method, params, invocation| {
            let st = state_for_methods.borrow();
            match method {
                "Play" => {
                    let path: String = params.child_value(0).get().unwrap_or_default();
                    println!("[aurora-player] DBus Play: {path}");
                    player::play(&st, &path);
                    invocation.return_value(None);
                }
                "Pause" => {
                    player::pause(&st);
                    invocation.return_value(None);
                }
                "Stop" => {
                    player::stop(&st);
                    invocation.return_value(None);
                }
                _ => {
                    invocation.return_error(
                        gio::IOErrorEnum::NotSupported,
                        &format!("Unknown method: {method}"),
                    );
                }
            }
        },
        |_conn, _sender, _path, _interface, _prop| glib::Variant::from(""),
        |_conn, _sender, _path, _interface, _prop, _value| false,
    );

    match reg_id {
        Ok(_) => println!("[aurora-player] DBus object registered"),
        Err(e) => eprintln!("[aurora-player] DBus registration error: {e}"),
    }

    connection
        .call_future(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus",
            "RequestName",
            Some(&glib::Variant::tuple_from_iter([
                glib::Variant::from("dev.daniacosta.AuroraWall.Player"),
                glib::Variant::from(0u32),
            ])),
            None,
            gio::DBusCallFlags::NONE,
            5000,
        )
        .await
        .map(|_| println!("[aurora-player] DBus name acquired"))
        .unwrap_or_else(|e| eprintln!("[aurora-player] DBus name request failed: {e}"));
}