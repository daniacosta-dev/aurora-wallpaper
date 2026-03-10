#!/bin/bash
set -e

INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

cp aurora-wallpaper "$INSTALL_DIR/"
cp aurora-player "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/aurora-wallpaper"
chmod +x "$INSTALL_DIR/aurora-player"

# Add to PATH if not already there
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.zshrc" 2>/dev/null || true
    echo "Added ~/.local/bin to PATH — restart your terminal"
fi

echo "Aurora Video Wallpaper installed successfully!"
echo "Run: aurora-wallpaper"
