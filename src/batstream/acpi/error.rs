// Copyright 2023 developers of the `batmon` project
// SPDX-License-Identifier: MPL-2.0

use netlink_packet_generic::{ctrl::GenlCtrl, GenlMessage};
use thiserror::Error;

use super::Msg;

#[derive(Error, Debug)]
pub enum Error {
    #[error("netlink error: {0}")]
    NetlinkCtrl(#[from] netlink_proto::Error<GenlMessage<GenlCtrl>>),
    #[error("netlink error: {0}")]
    NetlinkAcpi(#[from] netlink_proto::Error<Msg>),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
