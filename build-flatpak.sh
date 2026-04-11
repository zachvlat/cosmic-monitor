#!/bin/bash
# Build cosmicfetch flatpak package
# 
# System dependencies needed to BUILD (install with apt):
#   sudo apt install -y \
#     pkg-config libssl-dev libudev-dev libgtk-3-dev libfontconfig1-dev \
#     libfreetype6-dev libpng-dev zlib1g-dev libxkbcommon-dev libwayland-dev \
#     libegl-dev libgl-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev \
#     libsystemd-dev libjson-glib-dev liblzma-dev liblz4-dev libzstd-dev
#
# Runtime dependencies needed on target system:
#   org.freedesktop.Platform 24.08
#   org.freedesktop.Sdk 24.08

set -e
cd "$(dirname "$0")"

# Add flathub remote if not exists
flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo || true

RUNTIME_VERSION="24.08"

# Install runtime if needed
flatpak install -y flathub "org.freedesktop.Platform/x86_64/${RUNTIME_VERSION}" "org.freedesktop.Sdk/x86_64/${RUNTIME_VERSION}" || true

# Clean previous builds
rm -rf .flatpak-builder/repo /tmp/cosmicfetch-build /tmp/build-dir

# Build release binary
cargo build --release

# Create repo for flatpak
mkdir -p .flatpak-builder/repo
ostree --repo=.flatpak-builder/repo init --mode=archive

# Initialize flatpak build dir with runtime
flatpak build-init /tmp/build-dir com.zachvlat.cosmicfetch org.freedesktop.Platform/x86_64/${RUNTIME_VERSION} org.freedesktop.Sdk/x86_64/${RUNTIME_VERSION}

# Create directory structure
mkdir -p /tmp/build-dir/files/{bin,share/applications,share/metainfo,share/icons/hicolor/scalable/apps}

# Copy binary
cp target/release/cosmicfetch /tmp/build-dir/files/bin/

# Copy resources
cp resources/app.desktop /tmp/build-dir/files/share/applications/com.zachvlat.cosmicfetch.desktop
cp resources/app.metainfo.xml /tmp/build-dir/files/share/metainfo/com.zachvlat.cosmicfetch.metainfo.xml
cp resources/icons/hicolor/scalable/apps/com.zachvlat.cosmicfetch.svg /tmp/build-dir/files/share/icons/hicolor/scalable/apps/com.zachvlat.cosmicfetch.svg

# Finalize with permissions (filesystem=host:ro for accessing host commands like flatpak, dpkg)
flatpak build-finish /tmp/build-dir --command=cosmicfetch \
    --share=ipc \
    --socket=fallback-x11 \
    --socket=wayland \
    --device=dri \
    --share=network \
    --filesystem=host:ro \
    --filesystem=/run/host/usr/bin:ro \
    --filesystem=/run/host/usr/sbin:ro \
    --filesystem=/run/host/bin:ro \
    --filesystem=/run/host/sbin:ro \
    --filesystem=/usr/bin:ro \
    --filesystem=/usr/sbin:ro \
    --filesystem=/sys:ro \
    --talk-name=org.freedesktop.Flatpak

# Fix metadata to use correct runtime
cat > /tmp/build-dir/metadata << 'METADATA'
[Application]
name=com.zachvlat.cosmicfetch
runtime=org.freedesktop.Platform/x86_64/24.08
runtime-version=24.08
sdk=org.freedesktop.Sdk/x86_64/24.08
command=cosmicfetch

[Context]
shared=network;ipc;
sockets=x11;wayland;fallback-x11;
devices=dri;
filesystems=/run/host/bin:ro;/run/host/sbin:ro;/run/host/usr/bin:ro;/run/host/usr/sbin:ro;/usr/bin:ro;/usr/sbin:ro;/sys:ro;host:ro;

[Session Bus Policy]
org.freedesktop.Flatpak=talk
METADATA

# Export to repo and create bundle
flatpak build-export .flatpak-builder/repo /tmp/build-dir
flatpak build-bundle .flatpak-builder/repo cosmicfetch.flatpak com.zachvlat.cosmicfetch

echo "Done: cosmicfetch.flatpak"

# To install on another PC:
#   flatpak install cosmicfetch.flatpak
#   flatpak run com.zachvlat.cosmicfetch