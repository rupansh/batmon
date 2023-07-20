// Copyright 2023 developers of the `batmon` project
// SPDX-License-Identifier: MPL-2.0

use std::time::Duration;

use futures_lite::Future;
use notify_rust::{error::Error as NotifyError, Notification, Urgency};

use crate::{
    batstream::{AdapterStatus, BatEvent},
    priority::EvPriority,
};

use super::EvConsumer;

const NOTIFICATION_TIMOUT: Duration = Duration::from_secs(5);

struct EvInfo {
    summary: &'static str,
    body: String,
    urgency: Urgency,
}

impl From<super::Notification> for EvInfo {
    fn from(value: super::Notification) -> Self {
        match (value.event, value.priority) {
            (BatEvent::Battery(lvl), EvPriority::Low) => Self {
                summary: "Low Battery",
                body: format!("Battery level is low at {}%", lvl),
                urgency: Urgency::Low,
            },
            (BatEvent::Battery(lvl), EvPriority::Normal) => Self {
                summary: "Low Battery",
                body: format!("Battery level is low at {}%", lvl),
                urgency: Urgency::Normal,
            },
            (BatEvent::Battery(lvl), EvPriority::High) => Self {
                summary: "Critical Battery",
                body: format!("Battery level is critical at {}%", lvl),
                urgency: Urgency::Critical,
            },
            (BatEvent::Adapter(AdapterStatus::Connected), _) => Self {
                summary: "AC Adapter Connected",
                body: "AC Adapter has been connected".into(),
                urgency: Urgency::Low,
            },
            (BatEvent::Adapter(AdapterStatus::Disconnected), _) => Self {
                summary: "AC Adapter Disconnected",
                body: "AC Adapter has been disconnected".into(),
                urgency: Urgency::Low,
            },
        }
    }
}

pub struct NotifyConsumer {
    appname: String,
}

impl NotifyConsumer {
    pub fn new(appname: String) -> Self {
        Self { appname }
    }
}

pub type NotifFut = impl Future<Output = Result<(), NotifyError>>;

fn show_notification(notif: Notification) -> NotifFut {
    async move {
        _ = notif.show_async().await?;
        Ok(())
    }
}

impl EvConsumer for NotifyConsumer {
    type Error = notify_rust::error::Error;
    type Res<'a> = NotifFut;

    fn consume(&self, notif: super::Notification) -> NotifFut {
        let info = EvInfo::from(notif);
        let notif = Notification::new()
            .appname(&self.appname)
            .summary(info.summary)
            .body(&info.body)
            .urgency(info.urgency)
            .timeout(NOTIFICATION_TIMOUT)
            .finalize();
        show_notification(notif)
    }
}
