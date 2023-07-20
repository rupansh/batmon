#![feature(type_alias_impl_trait)]
// Copyright 2023 developers of the `batmon` project
// SPDX-License-Identifier: MPL-2.0


use std::{error::Error, time::Duration};

use args::Args;
use batstream::{polling::PollingStream, udev::UdevStream, BatEvent};
use clap::Parser;
use futures_lite::{Stream, StreamExt};
use notif::notify::NotifyConsumer;
use priority::PriorityThreshold;

use crate::{notif::Notification, priority::EvPriority};

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

    while let Some(event) = stream.next().await.transpose().unwrap() {
        let priority = if let BatEvent::Battery(lvl) = event {
            threshold.priority(lvl)
        } else {
            Some(EvPriority::Low)
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
    let args = Args::parse();
    let consumer = NotifyConsumer::new("batmon".into());
    let threshold = PriorityThreshold {
        low: args.low,
        normal: args.very_low,
        high: args.critical,
    };

    match args.backend {
        args::Backend::Polling => {
            let interval = Duration::from_secs(args.polling_interval);
            let stream = PollingStream::new(interval, args.battery, args.adapter)
                .await
                .unwrap();
            stream_loop(stream, consumer, threshold).await;
        }
        args::Backend::Udev => {
            let stream = UdevStream::new(args.battery, args.adapter).unwrap();
            stream_loop(stream, consumer, threshold).await;
        }
    }
}
