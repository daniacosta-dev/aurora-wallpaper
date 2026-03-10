// AuroraWall GNOME Shell Extension
// Listens on DBus for video path commands from the Rust app.
// Renders the video behind the desktop using a GStreamer pipeline + Clutter.

import GLib from 'gi://GLib';
import Gio from 'gi://Gio';
import GObject from 'gi://GObject';
import St from 'gi://St';
import Clutter from 'gi://Clutter';
import GstClutter from 'gi://GstClutter';
import Gst from 'gi://Gst';
import Meta from 'gi://Meta';

import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';

// DBus interface exposed by the extension.
// The Rust app calls SetWallpaper(path) to activate a video.
const DBUS_INTERFACE = `
<node>
  <interface name="dev.daniacosta.AuroraWall">
    <method name="SetWallpaper">
      <arg type="s" direction="in" name="path"/>
    </method>
    <method name="ClearWallpaper">
    </method>
    <property name="ActivePath" type="s" access="read"/>
  </interface>
</node>`;

const DBUS_NAME = 'dev.daniacosta.AuroraWall';
const DBUS_PATH = '/dev/daniacosta/AuroraWall';

export default class AuroraWallExtension extends Extension {
    enable() {
        // Init GStreamer.
        Gst.init(null);
        GstClutter.init(null);

        this._activePath = '';
        this._actor = null;
        this._pipeline = null;

        // Register DBus service.
        this._dbusImpl = Gio.DBusExportedObject.wrapJSObject(
            DBUS_INTERFACE,
            this
        );
        this._dbusImpl.export(Gio.DBus.session, DBUS_PATH);

        this._busNameId = Gio.bus_own_name(
            Gio.BusType.SESSION,
            DBUS_NAME,
            Gio.BusNameOwnerFlags.NONE,
            null, null, null
        );

        console.log('[AuroraWall] Extension enabled, DBus registered');
    }

    disable() {
        this._clearWallpaper();

        if (this._busNameId) {
            Gio.bus_unown_name(this._busNameId);
            this._busNameId = null;
        }

        if (this._dbusImpl) {
            this._dbusImpl.unexport();
            this._dbusImpl = null;
        }

        console.log('[AuroraWall] Extension disabled');
    }

    // --- DBus methods ---

    SetWallpaper(path) {
        console.log(`[AuroraWall] SetWallpaper: ${path}`);
        this._activePath = path;
        this._startVideo(path);
    }

    ClearWallpaper() {
        console.log('[AuroraWall] ClearWallpaper');
        this._activePath = '';
        this._clearWallpaper();
    }

    // DBus property.
    get ActivePath() {
        return this._activePath;
    }

    // --- Internal ---

    _startVideo(path) {
        this._clearWallpaper();

        const uri = GLib.filename_to_uri(path, null);

        // Create a GstClutter.VideoTexture actor to render video.
        this._actor = new GstClutter.VideoTexture();

        // Size to cover the primary monitor.
        const monitor = Main.layoutManager.primaryMonitor;
        this._actor.set_size(monitor.width, monitor.height);
        this._actor.set_position(monitor.x, monitor.y);

        // Place behind all other shell actors.
        global.window_group.insert_child_below(this._actor, null);

        // Connect to the underlying pipeline.
        this._pipeline = this._actor.get_pipeline();

        // Loop the video.
        this._actor.connect('eos', () => {
            this._actor.set_uri(uri);
            this._actor.set_playing(true);
        });

        this._actor.set_uri(uri);
        this._actor.set_playing(true);
    }

    _clearWallpaper() {
        if (this._actor) {
            this._actor.set_playing(false);
            this._actor.get_parent()?.remove_child(this._actor);
            this._actor = null;
            this._pipeline = null;
        }
    }
}