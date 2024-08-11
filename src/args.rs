// Copyright 2023 developers of the `batmon` project
// SPDX-License-Identifier: MPL-2.0

use std::{fs, path::PathBuf};

use clap::{Parser, ValueEnum};
use serde::Deserialize;

use crate::batstream::BatLvl;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The configuration file to use
    #[clap(short, long, default_value = "config.toml")]
    pub config: Config,

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
    Acpi,
}

#[derive(Default, Debug, Deserialize, Clone)]
pub struct Config {
    pub battery: Option<String>,
    pub adapter: Option<String>,
    pub low: Option<BatLvl>,
    pub very_low: Option<BatLvl>,
    pub critical: Option<BatLvl>,
    pub backend: Option<Backend>,
    pub polling_interval: Option<u64>,
}

impl Args {
    pub fn merge_with_config(mut self, config: Config) -> Self {
        if let Some(battery) = config.battery {
            self.battery = battery;
        }
        if let Some(adapter) = config.adapter {
            self.adapter = adapter;
        }
        if let Some(low) = config.low {
            self.low = low;
        }
        if let Some(very_low) = config.very_low {
            self.very_low = very_low;
        }
        if let Some(critical) = config.critical {
            self.critical = critical;
        }
        if let Some(backend) = config.backend {
            self.backend = backend;
        }
        if let Some(polling_interval) = config.polling_interval {
            self.polling_interval = polling_interval;
        }
        self
    }

    pub fn load_config(&self) -> Config {
        let home_config_path = dirs::home_dir().map(|home| home.join(".config/batmon/config.toml"));
        let system_config_path = PathBuf::from("/etc/batmon/config.toml");

        let config_content = if let Some(home_config_path) = home_config_path {
            fs::read_to_string(home_config_path)
                .or_else(|_| fs::read_to_string(system_config_path))
                .unwrap_or_default()
        } else {
            fs::read_to_string(system_config_path).unwrap_or_default()
        };
        toml::from_str(&config_content).unwrap()
    }
}