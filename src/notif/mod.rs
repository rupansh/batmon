// Copyright 2023 developers of the `batmon` project
// SPDX-License-Identifier: MPL-2.0

use std::error::Error;

use futures_lite::Future;

use crate::{batstream::BatEvent, priority::EvPriority};

pub mod notify;

pub struct Notification {
    event: BatEvent,
    priority: EvPriority,
}

impl Notification {
    pub fn new(event: BatEvent, priority: EvPriority) -> Self {
        Self { event, priority }
    }
}

pub trait EvConsumer {
    type Error: Error;
    type Res<'a>: Future<Output = Result<(), Self::Error>> + 'a
    where
        Self: 'a;
    fn consume(&self, notif: Notification) -> Self::Res<'_>;
}
