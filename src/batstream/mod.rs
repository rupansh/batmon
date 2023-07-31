// Copyright 2023 developers of the `batmon` project
// SPDX-License-Identifier: MPL-2.0

//! Battery Events Streams
pub mod acpi;
pub mod polling;
mod sysfs;
pub mod udev;
mod udev_bat;

use bounded_integer::BoundedU8;

pub type BatLvl = BoundedU8<0, 100>;

#[derive(Debug, Clone, Copy)]
pub enum AdapterStatus {
    Connected,
    Disconnected,
}

#[derive(Debug, Clone, Copy)]
pub enum BatEvent {
    Adapter(AdapterStatus),
    Battery(BatLvl),
}
