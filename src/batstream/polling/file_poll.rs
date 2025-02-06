// Copyright 2023 developers of the `batmon` project
// SPDX-License-Identifier: MPL-2.0

//! File Polling utilities
use std::time::Duration;

use async_stream::stream;
use futures_lite::Stream;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt},
    time,
};

pub async fn read_file(file: &mut File) -> Vec<u8> {
    let mut out = Vec::new();
    out.reserve_exact(6);
    file.read_to_end(&mut out).await.expect("failed to read fd");
    file.rewind().await.expect("failed to rewind fd");
    out
}

pub fn file_poller_cache(interval: Duration, mut file: File) -> impl Stream<Item = Vec<u8>> {
    stream! {
        let mut cache = Vec::new();
        loop {
            time::sleep(interval).await;
            let out = read_file(&mut file).await;
            if cache != out {
                cache = out.clone();
                yield out;
            }
        }
    }
}
