#!/bin/bash
set -e

echo "Aurora Video Wallpaper - Installer"
echo "==================================="
echo ""

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
    fi
}

install_deps

INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"
echo ""
echo "Installing binaries to $INSTALL_DIR..."
cp aurora-wallpaper "$INSTALL_DIR/"
cp aurora-player "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/aurora-wallpaper"
chmod +x "$INSTALL_DIR/aurora-player"

if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.zshrc" 2>/dev/null || true
fi

ICON_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"
mkdir -p "$ICON_DIR"
cp aurora-wallpaper.svg "$ICON_DIR/aurora-wallpaper.svg"
gtk-update-icon-cache ~/.local/share/icons/hicolor/ 2>/dev/null || true

DESKTOP_DIR="$HOME/.local/share/applications"
mkdir -p "$DESKTOP_DIR"

EXEC_PATH="$HOME/.local/bin/aurora-wallpaper"
cat > "$DESKTOP_DIR/aurora-wallpaper.desktop" << DESK
[Desktop Entry]
Name=Aurora Video Wallpaper
Comment=Animated video wallpaper manager for GNOME
Exec=$EXEC_PATH
Icon=aurora-wallpaper
Type=Application
Categories=Utility;GTK;
Keywords=wallpaper;video;animated;background;
DESK

echo "Desktop entry created — Aurora Video Wallpaper will appear in your app launcher."
echo ""
echo "✅ Aurora Video Wallpaper installed successfully!"
echo ""
echo "Launch from your app launcher, or run:"
echo "   $HOME/.local/bin/aurora-wallpaper"
