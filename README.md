# Batmon

[![RUST](https://img.shields.io/badge/made%20with-RUST-red.svg?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![MPL-2.0](https://img.shields.io/badge/license%20-MPL--2.0-white.svg?style=for-the-badge&logo=mozilla)](https://spdx.org/licenses/MPL-2.0.html)

Just another battery monitor for Linux.

## Why

ACPI events doesn't work properly for my laptop, and most of the polling implementations look pretty boring.  
Who doesn't like messing with Rust futures?  
I ended up adding other backends too.

## Backends

### Udev

This is the default backend, this should be fine for most laptops

### ACPI

Subscribe to the kernel's netlink socket for ACPI events, This may not work if udev events aren't working for you, but worth trying

### Polling

Use this if neither Udev nor Acpi backends work for you. 

## Service

A systemd service `batmon.service` has been provided. Feel free to modify it to change thresholds or other settings.

```bash
cargo install --path .
sudo install -m 0644 batmon.service /lib/systemd/user/
systemctl --user enable --now batmon.service
```

## Usage

```bash
batmon --help
```