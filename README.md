# System Monitor

A system monitor application for COSMIC desktop.

## Features

- **Overview** - CPU, memory, and process summary
- **CPU** - Per-core usage and frequency information
- **Memory** - Used/available/total with visual progress bar
- **Processes** - Top 20 processes by CPU usage
- **Network** - Interface statistics and total data transferred
- **Disks** - Mounted disk usage with progress bars

## Building

```bash
cargo build --release
```

## Running

```bash
./target/release/system-monitor
```

## Flatpak

Build the Flatpak:
```bash
cd flatpak
flatpak-builder --force-clean --repo=repo build com.zachvlat.test.yml
flatpak build-bundle repo system-monitor.flatpak com.zachvlat.system-monitor
```

Install:
```bash
flatpak install system-monitor.flatpak
```

## Dependencies

- libcosmic
- sysinfo
