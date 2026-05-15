#!/usr/bin/env bash
set -euo pipefail

NAME="$(basename "$(dirname "$(readlink -f "$0")")")"
SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"

YML="$SCRIPT_DIR/com.zachvlat.cosmicfetch.yml"

APP_ID="$(grep '^app-id:' "$YML" | awk '{print $2}')"
RUNTIME="$(grep '^runtime:' "$YML" | head -1 | awk '{print $2}')"
RUNTIME_VER="$(grep 'runtime-version:' "$YML" | awk '{print $2}' | tr -d '"')"
SDK="$(grep '^sdk:' "$YML" | awk '{print $2}')"

if [ -z "$APP_ID" ] || [ -z "$RUNTIME" ] || [ -z "$RUNTIME_VER" ] || [ -z "$SDK" ]; then
    echo "Failed to parse manifest" >&2
    exit 1
fi

BUILD_DIR="/tmp/opencode/${NAME}-flatpak-build"
REPO_DIR="/tmp/opencode/${NAME}-flatpak-repo"
OUTPUT="$SCRIPT_DIR/${NAME}.flatpak"

cleanup() {
    rm -rf "$BUILD_DIR" "$REPO_DIR"
}
trap cleanup EXIT

echo "==> Checking prerequisites..."
for cmd in flatpak cargo rsync install; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
        echo "Missing: $cmd" >&2
        exit 1
    fi
done
echo "    all found"

echo "==> Installing runtimes..."
flatpak install --user -y flathub \
    "$RUNTIME//$RUNTIME_VER" \
    "$SDK//$RUNTIME_VER" \
    "org.freedesktop.Sdk.Extension.rust-stable//$RUNTIME_VER" \
    2>&1 | sed 's/^/    /'

echo "==> Initializing flatpak build directory..."
rm -rf "$BUILD_DIR" "$REPO_DIR"
flatpak build-init "$BUILD_DIR" "$APP_ID" "$SDK//$RUNTIME_VER" "$RUNTIME//$RUNTIME_VER"

echo "==> Copying source..."
mkdir -p "$BUILD_DIR/files/project"
rsync -a --exclude=target --exclude=.git "$SCRIPT_DIR/" "$BUILD_DIR/files/project/"

echo "==> Building inside flatpak sandbox (this will take a while)..."
flatpak build --share=network "$BUILD_DIR" \
    sh -c 'export PATH="/usr/lib/sdk/rust-stable/bin:$PATH" && cd /app/project && cargo build --release --locked --manifest-path=Cargo.toml' \
    2>&1 | sed 's/^/    /'

echo "==> Installing files..."
install -Dm755 "$BUILD_DIR/files/project/target/release/cosmicfetch" "$BUILD_DIR/files/bin/cosmicfetch"
install -Dm644 "$SCRIPT_DIR/resources/app.desktop" "$BUILD_DIR/files/share/applications/${APP_ID}.desktop"
install -Dm644 "$SCRIPT_DIR/resources/app.metainfo.xml" "$BUILD_DIR/files/share/metainfo/${APP_ID}.metainfo.xml"
install -Dm644 "$SCRIPT_DIR/resources/icons/hicolor/scalable/apps/${APP_ID}.svg" \
    "$BUILD_DIR/files/share/icons/hicolor/scalable/apps/${APP_ID}.svg"
rm -rf "$BUILD_DIR/files/project"

echo "==> Running build-finish..."
flatpak build-finish "$BUILD_DIR" \
    --device=dri \
    --share=ipc \
    --share=network \
    --socket=wayland \
    --socket=fallback-x11 \
    --filesystem=host:ro \
    --filesystem=/var/lib/snapd:ro \
    --env=PATH=/run/host/usr/bin:/run/host/bin:/usr/bin:/bin \
    --talk-name=org.freedesktop.Flatpak \
    --command=/app/bin/cosmicfetch

echo "==> Exporting..."
flatpak build-export "$REPO_DIR" "$BUILD_DIR"

echo "==> Creating bundle..."
flatpak build-bundle "$REPO_DIR" "$OUTPUT" "$APP_ID"

echo ""
echo "Done! Flatpak created: $OUTPUT"
echo "Install with: flatpak install $OUTPUT"
