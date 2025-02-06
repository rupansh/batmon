// Copyright 2023 developers of the `batmon` project
// SPDX-License-Identifier: MPL-2.0

use std::{error::Error, time::Duration};

use args::Args;
use batstream::{acpi::AcpiStream, polling::polling_stream, udev::UdevStream, BatEvent};
use clap::Parser;
use futures_lite::{Stream, StreamExt};
// use notif::notify::NotifyConsumer;
use priority::PriorityThreshold;

use crate::{batstream::AdapterStatus, notif::Notification, priority::EvPriority};

mod args;
mod batstream;
mod notif;
mod priority;

/// Handle battery events
async fn stream_loop<E: Error>(
    stream: impl Stream<Item = Result<BatEvent, E>>,
    consumer: impl notif::EvConsumer,
    threshold: PriorityThreshold,
) {
    tokio::pin!(stream);

    let mut adapter_connected = false;
    let mut prev_bat_prio = None;
    while let Some(event) = stream.next().await.transpose().unwrap() {
        let priority = match event {
            BatEvent::Battery(lvl) if !adapter_connected => {
                let prio = threshold.priority(lvl);
                // Skip if we've already sent a notification with the same priority
                if prio == prev_bat_prio {
                    continue;
                }
                prev_bat_prio = prio;
                prio
            }
            BatEvent::Adapter(status) => {
                adapter_connected = status == AdapterStatus::Connected;
                prev_bat_prio = None;
                Some(EvPriority::Low)
            }
            _ => None,
        };
        let Some(priority) = priority else {
            continue;
        };

        let notif = Notification::new(event, priority);

        consumer.consume(notif).await.unwrap();
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();
    let consumer = {
        #[cfg(feature = "mock-notifications")]
        {
            notif::logger::LoggerNotifier
        }
        #[cfg(not(feature = "mock-notifications"))]
        {
            use notif::notify::NotifyConsumer;
            NotifyConsumer::new("batmon".into())
        }
    };
    let threshold = PriorityThreshold {
        low: args.low,
        normal: args.very_low,
        high: args.critical,
    };

    match args.backend {
        args::Backend::Polling => {
            let interval = Duration::from_secs(args.polling_interval);
            let stream = polling_stream(interval, args.battery, args.adapter)
                .await
                .unwrap();
            stream_loop(stream, consumer, threshold).await;
        }
        args::Backend::Udev => {
            let stream = UdevStream::new(args.battery, args.adapter).unwrap();
            stream_loop(stream, consumer, threshold).await;
        }
        args::Backend::Acpi => {
            let stream = AcpiStream::new(&args.battery).await.unwrap();
            stream_loop(stream, consumer, threshold).await;
        }
    }
}
