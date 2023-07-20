// Copyright 2023 developers of the `batmon` project
// SPDX-License-Identifier: MPL-2.0

//! SysFS constants
use const_format::concatcp;

/// Battery Subsystem
pub const BAT_SUBSYS: &str = "power_supply";
/// Base path to Battery Subsystem class
pub const BAT_BASE_PATH: &str = concatcp!("/sys/class/", BAT_SUBSYS);
