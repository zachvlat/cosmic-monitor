#!/bin/bash
set -e
cd "$(dirname "$0")"

rm -rf .flatpak-builder/repo /tmp/cosmic-monitor-build /tmp/build-dir

cargo build --release

mkdir -p .flatpak-builder/repo
ostree --repo=.flatpak-builder/repo init --mode=archive

flatpak build-init /tmp/build-dir com.zachvlat.cosmic-monitor org.freedesktop.Platform org.freedesktop.Sdk
mkdir -p /tmp/build-dir/files/{bin,share/applications,share/metainfo,share/icons/hicolor/scalable/apps}
cp target/release/cosmic-monitor /tmp/build-dir/files/bin/
cp resources/app.desktop /tmp/build-dir/files/share/applications/com.zachvlat.cosmic-monitor.desktop
cp resources/app.metainfo.xml /tmp/build-dir/files/share/metainfo/com.zachvlat.cosmic-monitor.metainfo.xml
cp resources/icons/hicolor/scalable/apps/icon.svg /tmp/build-dir/files/share/icons/hicolor/scalable/apps/com.zachvlat.cosmic-monitor.svg

flatpak build-finish /tmp/build-dir --command=cosmic-monitor \
    --share=ipc --socket=fallback-x11 --socket=wayland \
    --device=dri --share=network --filesystem=host

flatpak build-export .flatpak-builder/repo /tmp/build-dir
flatpak build-bundle .flatpak-builder/repo cosmic-monitor.flatpak com.zachvlat.cosmic-monitor

echo "Done: cosmic-monitor.flatpak"
