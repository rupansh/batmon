// Copyright 2023 developers of the `batmon` project
// SPDX-License-Identifier: MPL-2.0

use clap::{Parser, ValueEnum};

use crate::batstream::BatLvl;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The battery device to monitor
    #[clap(short, long, default_value = "BAT0")]
    pub battery: String,

    /// The adapter device to monitor
    #[clap(short, long, default_value = "ACAD")]
    pub adapter: String,

    /// The threshold for low battery
    #[clap(long, default_value = "30", value_name = "LEVEL")]
    pub low: BatLvl,

    /// The threshold for very low battery
    #[clap(long, default_value = "15", value_name = "LEVEL")]
    pub very_low: BatLvl,

    /// The threshold for critical battery
    #[clap(long, default_value = "8", value_name = "LEVEL")]
    pub critical: BatLvl,

    /// The backend to use for fetching power data.
    #[clap(long, default_value = "udev")]
    pub backend: Backend,

    /// The polling interval in seconds,
    /// only applicable for --backend polling
    #[clap(long, default_value = "5", value_name = "SECONDS")]
    pub polling_interval: u64,
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, ValueEnum)]
pub enum Backend {
    Udev,
    Polling,
}
