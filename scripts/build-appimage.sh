#!/usr/bin/env bash
set -euo pipefail

APP_ID="io.github.dalpat.dockermanager"
APP_NAME="docker-manager"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$PROJECT_ROOT/dist"
TOOLS_DIR="$PROJECT_ROOT/.tools"
APPDIR="$DIST_DIR/$APP_ID.AppDir"

ARCH="$(uname -m)"
if [[ "$ARCH" != "x86_64" ]]; then
  echo "Unsupported architecture for this script: $ARCH"
  echo "Supported architecture: x86_64"
  exit 1
fi

mkdir -p "$DIST_DIR" "$TOOLS_DIR"

LINUXDEPLOY_APPIMAGE="$TOOLS_DIR/linuxdeploy-x86_64.AppImage"
GTK_PLUGIN_APPIMAGE="$TOOLS_DIR/linuxdeploy-plugin-gtk-x86_64.AppImage"

download_if_missing() {
  local url="$1"
  local output="$2"
  if [[ ! -s "$output" ]]; then
    rm -f "$output"
    echo "Downloading $(basename "$output")..."
    curl -fsSL "$url" -o "$output"
    chmod +x "$output"
  fi
}

download_optional() {
  local url="$1"
  local output="$2"
  if [[ -s "$output" ]]; then
    return 0
  fi

  rm -f "$output"
  echo "Attempting optional download: $(basename "$output")"
  if curl -fsSL "$url" -o "$output"; then
    chmod +x "$output"
  else
    rm -f "$output"
    echo "Optional tool unavailable: $(basename "$output")"
  fi
}

download_if_missing \
  "https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage" \
  "$LINUXDEPLOY_APPIMAGE"

# Optional: plugin URL availability changes over time. Build continues without it.
download_optional \
  "https://github.com/linuxdeploy/linuxdeploy-plugin-gtk/releases/download/continuous/linuxdeploy-plugin-gtk-x86_64.AppImage" \
  "$GTK_PLUGIN_APPIMAGE"

echo "Building release binary..."
cargo build --release --manifest-path "$PROJECT_ROOT/Cargo.toml"

echo "Preparing AppDir..."
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"

install -m 755 "$PROJECT_ROOT/target/release/$APP_NAME" "$APPDIR/usr/bin/$APP_NAME"
install -m 644 \
  "$PROJECT_ROOT/packaging/$APP_ID.desktop" \
  "$APPDIR/$APP_ID.desktop"
install -m 644 \
  "$PROJECT_ROOT/assets/icons/$APP_ID.svg" \
  "$APPDIR/$APP_ID.svg"
mkdir -p "$APPDIR/usr/share/metainfo"
install -m 644 \
  "$PROJECT_ROOT/packaging/$APP_ID.appdata.xml" \
  "$APPDIR/usr/share/metainfo/$APP_ID.appdata.xml"

if [[ ! -f "$APPDIR/AppRun" ]]; then
  cat > "$APPDIR/AppRun" <<'EOF'
#!/usr/bin/env bash
HERE="$(dirname "$(readlink -f "$0")")"
exec "$HERE/usr/bin/docker-manager" "$@"
EOF
  chmod +x "$APPDIR/AppRun"
fi

export ARCH=x86_64
export LDAI_OUTPUT="$DIST_DIR/${APP_NAME}-linux-x86_64.AppImage"
export GTK_THEME="${GTK_THEME:-Adwaita:dark}"

echo "Bundling runtime dependencies with linuxdeploy..."
if [[ -s "$GTK_PLUGIN_APPIMAGE" ]]; then
  echo "GTK plugin detected, enabling enhanced GTK bundling."
  APPIMAGE_EXTRACT_AND_RUN=1 LINUXDEPLOY_PLUGIN=gtk "$LINUXDEPLOY_APPIMAGE" \
    --appdir "$APPDIR" \
    --desktop-file "$PROJECT_ROOT/packaging/$APP_ID.desktop" \
    --icon-file "$PROJECT_ROOT/assets/icons/$APP_ID.svg" \
    --executable "$PROJECT_ROOT/target/release/$APP_NAME" \
    --output appimage
else
  echo "GTK plugin not available; proceeding with linuxdeploy core bundling."
  APPIMAGE_EXTRACT_AND_RUN=1 "$LINUXDEPLOY_APPIMAGE" \
    --appdir "$APPDIR" \
    --desktop-file "$PROJECT_ROOT/packaging/$APP_ID.desktop" \
    --icon-file "$PROJECT_ROOT/assets/icons/$APP_ID.svg" \
    --executable "$PROJECT_ROOT/target/release/$APP_NAME" \
    --output appimage
fi

APPIMAGE_PATH="$LDAI_OUTPUT"
if [[ ! -f "$APPIMAGE_PATH" ]]; then
  APPIMAGE_PATH="$(find "$DIST_DIR" -maxdepth 1 -type f \( -name 'Docker_Manager-*.AppImage' -o -name 'docker-manager-*.AppImage' \) | head -n 1 || true)"
fi
if [[ -z "$APPIMAGE_PATH" ]]; then
  APPIMAGE_PATH="$(find "$DIST_DIR" -maxdepth 1 -type f -name '*.AppImage' | head -n 1 || true)"
fi
if [[ -z "$APPIMAGE_PATH" ]]; then
  echo "AppImage creation failed: no AppImage produced."
  exit 1
fi

FINAL_APPIMAGE="$DIST_DIR/${APP_NAME}-linux-x86_64.AppImage"
if [[ "$APPIMAGE_PATH" != "$FINAL_APPIMAGE" ]]; then
  mv -f "$APPIMAGE_PATH" "$FINAL_APPIMAGE"
fi
chmod +x "$FINAL_APPIMAGE"
sha256sum "$FINAL_APPIMAGE" > "$FINAL_APPIMAGE.sha256"

echo "Done."
echo "AppImage: $FINAL_APPIMAGE"
echo "Checksum: $FINAL_APPIMAGE.sha256"
