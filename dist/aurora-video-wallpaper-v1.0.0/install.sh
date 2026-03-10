#!/bin/bash
set -e

echo "Aurora Video Wallpaper - Installer"
echo "==================================="
echo ""

# ── Dependency installation ───────────────────────────────────────────────────

install_deps() {
    if command -v apt &>/dev/null; then
        echo "Detected apt — installing dependencies..."
        sudo apt install -y libmpv2 libgtk-4-1 libadwaita-1-0
    elif command -v dnf &>/dev/null; then
        echo "Detected dnf — installing dependencies..."
        sudo dnf install -y mpv-libs gtk4 libadwaita
    elif command -v pacman &>/dev/null; then
        echo "Detected pacman — installing dependencies..."
        sudo pacman -S --noconfirm mpv gtk4 libadwaita
    elif command -v zypper &>/dev/null; then
        echo "Detected zypper — installing dependencies..."
        sudo zypper install -y libmpv2 gtk4 libadwaita-1-0
    else
        echo "⚠  Could not detect package manager."
        echo "   Please install manually: libmpv, gtk4, libadwaita"
        echo ""
    fi
}

install_deps

# ── Install binaries ──────────────────────────────────────────────────────────

INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

echo ""
echo "Installing binaries to $INSTALL_DIR..."

cp aurora-wallpaper "$INSTALL_DIR/"
cp aurora-player "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/aurora-wallpaper"
chmod +x "$INSTALL_DIR/aurora-player"

# ── PATH setup ────────────────────────────────────────────────────────────────

if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.zshrc" 2>/dev/null || true
fi

# ── Desktop entry ─────────────────────────────────────────────────────────────

DESKTOP_DIR="$HOME/.local/share/applications"
mkdir -p "$DESKTOP_DIR"

cat > "$DESKTOP_DIR/aurora-wallpaper.desktop" << DESKTOP
[Desktop Entry]
Name=Aurora Video Wallpaper
Comment=Animated video wallpaper manager for GNOME
Exec=$HOME/.local/bin/aurora-wallpaper
Icon=video-display
Type=Application
Categories=Utility;GTK;
Keywords=wallpaper;video;animated;background;
DESKTOP

echo "Desktop entry created — Aurora Video Wallpaper will appear in your app launcher."

# ── Done ──────────────────────────────────────────────────────────────────────

echo ""
echo "✅ Aurora Video Wallpaper installed successfully!"
echo ""
echo "Launch from your app launcher, or run:"
echo "   $INSTALL_DIR/aurora-wallpaper"