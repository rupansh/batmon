// Copyright 2023 developers of the `batmon` project
// SPDX-License-Identifier: MPL-2.0

//! Polling based stream implementation
mod file_poll;
use std::{
    ffi::OsStr,
    io,
    os::unix::prelude::OsStrExt,
    path::Path,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures_lite::{ready, stream::Skip, Stream, StreamExt};
use pin_project_lite::pin_project;
use tokio::fs::File;

use self::file_poll::file_poller_cache;

use super::{sysfs::BAT_BASE_PATH, AdapterStatus, BatEvent, BatLvl};

pin_project! {
    pub struct PollingStream<FilePollS> {
        #[pin]
        battery_state: FilePollS,
        #[pin]
        adapter_state: Skip<FilePollS>,
    }
}

pub async fn polling_stream(
    interval: Duration,
    battery_device: impl AsRef<Path>,
    adapter_device: impl AsRef<Path>,
) -> io::Result<PollingStream<impl Stream<Item = Vec<u8>>>> {
    PollingStream::new(interval, battery_device, adapter_device, file_poller_cache).await
}

impl<S: Stream> PollingStream<S> {
    pub async fn new(
        interval: Duration,
        battery_device: impl AsRef<Path>,
        adapter_device: impl AsRef<Path>,
        stream_gen: fn(Duration, File) -> S,
    ) -> io::Result<Self> {
        let mut battery_path = Path::new(BAT_BASE_PATH).join(battery_device);
        battery_path.push("capacity");
        let mut adapter_path = Path::new(BAT_BASE_PATH).join(adapter_device);
        adapter_path.push("online");

        let battery = File::open(battery_path).await?;
        let adapter = File::open(adapter_path).await?;

        Ok(Self {
            battery_state: stream_gen(interval, battery),
            adapter_state: stream_gen(interval, adapter).skip(1),
        })
    }
}

fn parse_os_str(raw: &[u8]) -> &str {
    OsStr::from_bytes(raw)
        .to_str()
        .and_then(|s| s.strip_suffix('\n'))
        .expect("data is not valid utf8?!")
}

fn parse_battery(raw: Vec<u8>) -> BatLvl {
    parse_os_str(&raw)
        .parse()
        .expect("battery capacity is not a valid?!")
}

fn parse_adapter(raw: Vec<u8>) -> AdapterStatus {
    let dat = parse_os_str(&raw);
    if dat == "1" {
        AdapterStatus::Connected
    } else {
        AdapterStatus::Disconnected
    }
}

fn handle_item<T: Stream<Item = Vec<u8>>, R>(
    stream: Pin<&mut T>,
    cx: &mut Context<'_>,
    parser: impl FnOnce(Vec<u8>) -> R,
) -> Poll<Option<R>> {
    let item = ready!(stream.poll_next(cx));
    let Some(item) = item else {
        return Poll::Ready(None);
    };

    Poll::Ready(Some(parser(item)))
}

impl<S: Stream<Item = Vec<u8>>> Stream for PollingStream<S> {
    type Item = io::Result<BatEvent>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        if let Poll::Ready(adp) = handle_item(this.adapter_state, cx, |v| {
            BatEvent::Adapter(parse_adapter(v))
        }) {
            return Poll::Ready(Ok(adp).transpose());
        }

        if let Poll::Ready(bat) = handle_item(this.battery_state, cx, |v| {
            BatEvent::Battery(parse_battery(v))
        }) {
            return Poll::Ready(Ok(bat).transpose());
        }

        Poll::Pending
    }
}
