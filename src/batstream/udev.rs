// Copyright 2023 developers of the `batmon` project
// SPDX-License-Identifier: MPL-2.0

//! Udev based battery event stream
use std::{
    io,
    path::{Path, PathBuf},
    pin::Pin,
    task::{Context, Poll},
};

use futures_lite::{ready, Stream, StreamExt};
use tokio_udev::{AsyncMonitorSocket, Device, Event, EventType, MonitorBuilder};

use super::{
    sysfs::{BAT_BASE_PATH, BAT_SUBSYS},
    AdapterStatus, BatEvent, BatLvl,
};

/// Udev based battery event stream
pub struct UdevStream {
    /// Path to battery device
    battery_path: PathBuf,
    /// Path to adapter device
    adapter_path: PathBuf,
    /// For pushing the current battery level
    first_lvl: Option<BatLvl>,
    monitor: AsyncMonitorSocket,
}

impl UdevStream {
    pub fn new(battery_dev: impl AsRef<Path>, adapter_dev: impl AsRef<Path>) -> io::Result<Self> {
        let battery_path = Path::new(BAT_BASE_PATH).join(battery_dev);
        let adapter_path = Path::new(BAT_BASE_PATH).join(adapter_dev);

        let battery = Device::from_syspath(&battery_path)?;
        let _adapter = Device::from_syspath(&adapter_path)?;

        let monitor = MonitorBuilder::new()?
            .match_subsystem(BAT_SUBSYS)?
            .listen()?
            .try_into()?;
        let first_lvl = Self::handle_battery(&battery);

        Ok(Self {
            battery_path,
            adapter_path,
            first_lvl: Some(first_lvl),
            monitor,
        })
    }

    /// extract battery capacity
    fn handle_battery(ev: &Device) -> BatLvl {
        ev.property_value("POWER_SUPPLY_CAPACITY")
            .expect("battery does not advertise capacity?!")
            .to_str()
            .expect("battery capacity is not valid utf8?!")
            .parse::<BatLvl>()
            .expect("battery capacity is not a valid u8?!")
    }

    /// extract adapter status
    fn handle_adapter(ev: &Device) -> AdapterStatus {
        if ev
            .property_value("POWER_SUPPLY_ONLINE")
            .expect("adapter does not advertise online?!")
            == "1"
        {
            AdapterStatus::Connected
        } else {
            AdapterStatus::Disconnected
        }
    }

    /// Handle udev event
    /// ignores if not a battery or adapter related event
    fn handle_event(&self, event: Event) -> Option<BatEvent> {
        if event.event_type() != EventType::Change {
            return None;
        }

        if event.syspath() == self.battery_path {
            Some(BatEvent::Battery(Self::handle_battery(&event)))
        } else if event.syspath() == self.adapter_path {
            Some(BatEvent::Adapter(Self::handle_adapter(&event)))
        } else {
            None
        }
    }
}

impl Stream for UdevStream {
    type Item = io::Result<BatEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(lvl) = self.first_lvl.take() {
            return Poll::Ready(Some(Ok(BatEvent::Battery(lvl))));
        }

        let res = loop {
            let event = match ready!(self.monitor.poll_next(cx)) {
                Some(Ok(event)) => event,
                Some(Err(err)) => return Poll::Ready(Some(Err(err))),
                None => return Poll::Ready(None),
            };

            match self.handle_event(event) {
                Some(event) => break event,
                None => continue,
            }
        };

        Poll::Ready(Some(Ok(res)))
    }
}
