# Cosmic Monitor

A system monitor application for COSMIC desktop.

## Features

- **Overview** - System information and resource usage summary
- **CPU** - Per-core usage and frequency information
- **Memory** - Used/available/total with visual progress bar
- **Processes** - Top 20 processes with sorting by name, CPU, or memory
- **Network** - Total data downloaded and uploaded
- **Disks** - Mounted disk usage with progress bars

## Building

```bash
cargo build --release
```

## Running

```bash
./target/release/cosmic-monitor
```

## Flatpak

Build the Flatpak:
```bash
./build-flatpak.sh
```

This creates `cosmic-monitor.flatpak` which can be installed on other computers.

Install:
```bash
flatpak install cosmic-monitor.flatpak
```

## Dependencies

- libcosmic
- sysinfo
