# Batmon

[![RUST](https://img.shields.io/badge/made%20with-RUST-red.svg?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![MPL-2.0](https://img.shields.io/badge/license%20-MPL--2.0-white.svg?style=for-the-badge&logo=mozilla)](https://spdx.org/licenses/MPL-2.0.html)

Just another battery monitor for Linux.

## Why

acpi events don't work properly for my laptop, and most of the polling implementations look pretty boring.  
who doesn't like messing with Rust futures?  
I ended up adding other backends too

## Usage

```bash
batman --help