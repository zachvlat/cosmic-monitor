# Cosmicfetch

A system monitor application for COSMIC desktop.

<img width="1061" height="796" alt="1" src="https://github.com/user-attachments/assets/62f6191a-5e26-4d05-bb12-602a3977b313" />
<img width="1061" height="796" alt="2" src="https://github.com/user-attachments/assets/8f2e4c6d-5bf4-4bd4-9536-fc83f7d53391" />
<img width="1061" height="796" alt="3" src="https://github.com/user-attachments/assets/a9eea4db-063e-4c47-a707-96007e124ee8" />
<img width="1061" height="796" alt="4" src="https://github.com/user-attachments/assets/1478263a-20a4-4acb-a86f-ba338b5e552a" />



## Features

- **Overview** - System information and resource usage summary
- **CPU** - Per-core usage and frequency information
- **Memory** - Used/available/total with visual progress bar
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
