#!/usr/bin/env bash
set -euo pipefail

APP_ID="io.github.dalpat.dockermanager"
BIN_NAME="docker-manager"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "Building release binary..."
cargo build --release --manifest-path "$PROJECT_ROOT/Cargo.toml"

echo "Installing binary to ~/.local/bin/$BIN_NAME"
mkdir -p "$HOME/.local/bin"
install -m 755 "$PROJECT_ROOT/target/release/$BIN_NAME" "$HOME/.local/bin/$BIN_NAME"

echo "Installing desktop entry..."
mkdir -p "$HOME/.local/share/applications"
install -m 644 \
  "$PROJECT_ROOT/packaging/$APP_ID.desktop" \
  "$HOME/.local/share/applications/$APP_ID.desktop"

echo "Installing scalable app icon..."
mkdir -p "$HOME/.local/share/icons/hicolor/scalable/apps"
install -m 644 \
  "$PROJECT_ROOT/assets/icons/$APP_ID.svg" \
  "$HOME/.local/share/icons/hicolor/scalable/apps/$APP_ID.svg"

echo "Refreshing desktop database and icon cache (if available)..."
if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "$HOME/.local/share/applications" || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f "$HOME/.local/share/icons/hicolor" || true
fi

echo "Done."
echo "If icon does not appear immediately, log out/in or restart GNOME Shell."
