<div align="center">

<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />
<img src="https://img.shields.io/badge/GTK4-4A90D9?style=for-the-badge&logo=gtk&logoColor=white" />
<img src="https://img.shields.io/badge/Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black" />
<img src="https://img.shields.io/badge/License-MIT-green?style=for-the-badge" />

# 🌌 Aurora Wallpaper

**Animated video wallpaper manager for Linux & GNOME**

*Beautiful. Efficient. Open source.*

[Features](#features) · [Installation](#installation) · [Usage](#usage) · [Performance](#performance) · [Contributing](#contributing) · [Support](#support)

</div>

---
como 
## What is AuroraWall?

AuroraWall lets you set any video as your desktop wallpaper on Linux. Built with Rust and GTK4, it's designed to be fast, memory-efficient, and multi-monitor aware — without compromising your system's performance.

---

## Features

- 🎬 **Video wallpapers** — MP4, WebM, MOV, MKV support
- 🖥️ **Multi-monitor** — independent video per monitor, dynamic hotplug detection
- ⚡ **Hardware decoding** — GPU acceleration via `hwdec=auto-safe` (falls back to software automatically)
- 🗂️ **Wallpaper library** — import, manage, and persist your collection
- 🔌 **DBus API** — control playback programmatically from any app or script
- 🔄 **Loop playback** — seamless infinite loop
- 🧩 **GNOME Extension** — scaffold included for deeper shell integration
- 🌐 **Wayland-ready architecture** — X11 today, gtk4-layer-shell tomorrow

---

## Performance

AuroraWall is built with performance as a first-class concern. Here are real benchmarks running **3 monitors simultaneously** on a release build:

| Metric | AuroraWall (`vo=gpu`) | AuroraWall (`vo=x11`) |
|--------|----------------------|----------------------|
| CPU usage | **~1.2%** | ~2.9% |
| RAM (resident) | **~143 MB** | ~182 MB |
| Hardware decoding | ✅ Yes | ❌ No |

> Benchmarked on Ubuntu 24.04, GNOME 46, X11, 3 monitors (1920×1080 each).

Thanks to `libmpv` with `vo=gpu` and `hwdec=auto-safe`, AuroraWall offloads video decoding to your GPU when available — with automatic fallback to software rendering for systems without a dedicated GPU.

---

## Security

AuroraWall takes a minimal-privilege approach:

- **No network access** — the app never makes network requests
- **No telemetry** — zero data collection, ever
- **Local storage only** — your library is stored in `~/.local/share/aurorawall/` and never leaves your machine
- **Sandboxable** — architecture is designed with future Flatpak distribution in mind, which will enforce strict filesystem and network isolation
- **DBus scoped** — the player registers under `dev.daniacosta.AuroraWall.Player` with no elevated privileges
- **Open source** — every line of code is auditable

---

## Architecture

AuroraWall is a Cargo workspace with three crates:

```
aurora-wallpaper/
├── app/          # GTK4 + libadwaita UI — library management
├── player/       # Headless player process — one video wallpaper per monitor
└── shared/       # DBus constants shared between app and player
```

The app and player are **separate processes** communicating via DBus. This means:
- A crash in the player never takes down the UI
- The player can run independently, controlled by any DBus client
- Future support for autostart via systemd user units

---

## Installation

### Dependencies

```bash
sudo apt install \
  libgtk-4-dev \
  libadwaita-1-dev \
  libgstreamer1.0-dev \
  libgstreamer-plugins-base1.0-dev \
  libmpv-dev \
  mpv \
  libgdk4-x11-dev
```

### Build from source

```bash
git clone https://github.com/daniacosta-dev/aurora-wallpaper
cd aurora-wallpaper
cargo build --release
```

Binaries will be at:
- `target/release/aurora-wallpaper` — the UI app
- `target/release/aurora-player` — the player process

---

## Usage

### Launch the app

```bash
./target/release/aurora-wallpaper
```

1. Click **Import Video** to add a video to your library
2. Click **Set as Wallpaper** to activate it on all monitors

### DBus API

Control the player from any terminal or script:

```bash
# Play on all monitors
gdbus call --session \
  --dest dev.daniacosta.AuroraWall.Player \
  --object-path /dev/daniacosta/AuroraWall/Player \
  --method dev.daniacosta.AuroraWall.Player.Play \
  "/path/to/video.mp4"

# Play on a specific monitor (0-indexed)
gdbus call --session \
  --dest dev.daniacosta.AuroraWall.Player \
  --object-path /dev/daniacosta/AuroraWall/Player \
  --method dev.daniacosta.AuroraWall.Player.PlayOnMonitor \
  "/path/to/video.mp4" 1

# Pause / Resume / Stop
gdbus call --session --dest dev.daniacosta.AuroraWall.Player \
  --object-path /dev/daniacosta/AuroraWall/Player \
  --method dev.daniacosta.AuroraWall.Player.Pause

# List available monitors
gdbus call --session --dest dev.daniacosta.AuroraWall.Player \
  --object-path /dev/daniacosta/AuroraWall/Player \
  --method dev.daniacosta.AuroraWall.Player.GetMonitors
```

---

## Roadmap

- [ ] Auto-pause when a window is maximized on that monitor
- [ ] Persist active wallpaper across reboots
- [ ] Autostart via systemd user unit
- [ ] Volume control per wallpaper
- [ ] Wayland support via gtk4-layer-shell
- [ ] Flatpak distribution
- [ ] Lock screen integration

---

## Contributing

Contributions are welcome. Please open an issue before submitting large PRs so we can discuss the approach.

```bash
# Run the app in dev mode
cargo run -p aurora-wallpaper

# Run the player
cargo run -p aurora-player
```

---

## Support

If AuroraWall saves you from a boring desktop, consider supporting development:

<div align="center">

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/daniacostadev)

</div>

---

<div align="center">

Made with ❤️ and Rust · MIT License · [Ko-fi](https://ko-fi.com/daniacostadev)

</div>

Created by [@daniacosta-dev](https://github.com/daniacosta-dev)