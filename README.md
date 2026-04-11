# Cosmicfetch

A system monitor application for COSMIC desktop.

<img width="1197" height="848" alt="image" src="https://github.com/user-attachments/assets/07685567-14b8-468c-85d4-44e05e9fae1f" />


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
./target/release/cosmicfetch
```

## Flatpak

Build the Flatpak:
```bash
./build-flatpak.sh
```

This creates `cosmicfetch.flatpak` which can be installed on other computers.

Install:
```bash
flatpak install cosmicfetch.flatpak
```

## Dependencies

- libcosmic
- sysinfo
