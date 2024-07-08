// Copyright 2023 developers of the `batmon` project
// SPDX-License-Identifier: MPL-2.0

//! File Polling utilities
use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futs::*;
use futures_lite::{ready, Future, Stream};
use pin_project_lite::pin_project;
use tokio::fs::File;

type ReadOut = (File, Vec<u8>);

mod futs {
    use futures_lite::Future;
    use std::time::Duration;
    use tokio::{
        fs::File,
        io::{AsyncReadExt, AsyncSeekExt},
        time,
    };

    pub type ReadFut = impl Future<Output = super::ReadOut>;
    pub type SleepFut = impl Future<Output = File>;

    pub fn read_file(mut file: File) -> ReadFut {
        async {
            let mut out = Vec::new();
            out.reserve_exact(6);
            file.read_to_end(&mut out).await.expect("failed to read fd");
            file.rewind().await.expect("failed to rewind fd");
            (file, out)
        }
    }

    pub fn identity_sleep(interval: Duration, file: File) -> SleepFut {
        async move {
            time::sleep(interval).await;
            file
        }
    }
}

// Sleep N seconds, read the file, reset the state(go back to sleeping)
pin_project! {
    #[project = StateProj]
    enum FilePoller {
        Pending { #[pin] pinned: SleepFut, interval: Duration },
        Read { #[pin] pinned: ReadFut, interval: Duration },
    }
}

impl FilePoller {
    fn new(interval: Duration, file: File) -> Self {
        Self::Pending {
            pinned: identity_sleep(interval, file),
            interval,
        }
    }
}

macro_rules! handle_read {
    ($self:ident, $read:ident, $interval:ident, $cx:ident) => {{
        let (file, out) = ready!($read.poll($cx));
        let interval = *$interval;
        $self.set(FilePoller::new(interval, file));
        return Poll::Ready(out);
    }};
}

impl Future for FilePoller {
    type Output = Vec<u8>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().project();
        match this {
            StateProj::Pending { pinned, interval } => {
                let file = ready!(pinned.poll(cx));
                let fut = read_file(file);
                let interval = *interval;
                self.set(FilePoller::Read {
                    pinned: fut,
                    interval,
                });
                let StateProj::Read { pinned, interval } = self.as_mut().project() else {
                    unreachable!()
                };
                handle_read!(self, pinned, interval, cx);
            }
            StateProj::Read { pinned, interval } => handle_read!(self, pinned, interval, cx),
        }
    }
}

pin_project! {
    /// Cache the last read value
    /// only emits if the value changes
    pub struct FilePollerCache {
        cache: Vec<u8>,
        #[pin]
        poller: FilePoller,
    }
}

impl FilePollerCache {
    pub fn new(interval: Duration, file: File) -> Self {
        Self {
            cache: Vec::new(),
            poller: FilePoller::new(interval, file),
        }
    }
}

impl Stream for FilePollerCache {
    type Item = Vec<u8>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Vec<u8>>> {
        loop {
            let this = self.as_mut().project();
            let out = ready!(this.poller.poll(cx));
            if *this.cache != out {
                *this.cache = out.clone();
                break Poll::Ready(Some(out));
            }
        }
    }
}
