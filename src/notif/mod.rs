// Copyright 2023 developers of the `batmon` project
// SPDX-License-Identifier: MPL-2.0

use std::error::Error;

use crate::{batstream::BatEvent, priority::EvPriority};

#[cfg(feature = "mock-notifications")]
pub mod logger;
#[cfg(not(feature = "mock-notifications"))]
pub mod notify;

#[derive(Debug)]
pub struct Notification {
    event: BatEvent,
    priority: EvPriority,
}

impl Notification {
    pub fn new(event: BatEvent, priority: EvPriority) -> Self {
        Self { event, priority }
    }
}

pub(crate) trait EvConsumer {
    type Error: Error;

    async fn consume(&self, notif: Notification) -> Result<(), Self::Error>;
}
