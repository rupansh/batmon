// Copyright 2023 developers of the `batmon` project
// SPDX-License-Identifier: MPL-2.0

use std::{
    ffi::{c_char, CStr},
    str::FromStr,
};

use arrayvec::ArrayString;
use bytemuck::{must_cast_ref, AnyBitPattern};
use netlink_packet_generic::{GenlFamily, GenlHeader};
use netlink_packet_utils::{
    nla::{NlaBuffer, NlasIterator},
    DecodeError, Parseable, ParseableParametrized,
};

const ACPI_GENL_CMD_UNSPEC: u8 = 0;
const ACPI_GENL_CMD_EVENT: u8 = 1;

const ACPI_GENL_ATTR_UNSPEC: u16 = 0;
const ACPI_GENL_ATTR_EVENT: u16 = 1;

#[derive(Debug, Clone, Copy)]
pub enum AcpiGenlCmd {
    Unspec,
    Event,
}

impl From<AcpiGenlCmd> for u8 {
    fn from(value: AcpiGenlCmd) -> Self {
        match value {
            AcpiGenlCmd::Unspec => ACPI_GENL_CMD_UNSPEC,
            AcpiGenlCmd::Event => ACPI_GENL_CMD_EVENT,
        }
    }
}

impl TryInto<AcpiGenlCmd> for u8 {
    type Error = DecodeError;

    fn try_into(self) -> Result<AcpiGenlCmd, Self::Error> {
        match self {
            ACPI_GENL_CMD_UNSPEC => Ok(AcpiGenlCmd::Unspec),
            ACPI_GENL_CMD_EVENT => Ok(AcpiGenlCmd::Event),
            _ => Err(DecodeError::from("unknown ctrl cmd")),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AcpiGenlAttr {
    Unspec(Vec<u8>),
    Event(AcpiGenlEvent),
}

impl<'a, T: AsRef<[u8]> + ?Sized> Parseable<NlaBuffer<&'a T>> for AcpiGenlAttr {
    fn parse(buf: &NlaBuffer<&'a T>) -> Result<Self, DecodeError> {
        match buf.kind() {
            ACPI_GENL_ATTR_UNSPEC => Ok(Self::Unspec(buf.value().to_vec())),
            ACPI_GENL_ATTR_EVENT => Ok(Self::Event(AcpiGenlEvent::try_from(buf.value())?)),
            _ => Err(DecodeError::from("unknown acpi_genl_attr")),
        }
    }
}

#[derive(Clone, Copy, AnyBitPattern)]
#[repr(C)]
struct RawAcpiGenlEvent {
    device_class: [c_char; 20],
    bus_id: [c_char; 15],
    kind: u32,
    data: u32,
}

#[derive(Debug, Clone)]
pub struct AcpiGenlEvent {
    device_class: ArrayString<20>,
    pub kind: u32,
    pub data: u32,
}

impl AcpiGenlEvent {
    pub fn device_class(&self) -> &str {
        &self.device_class
    }
}

impl<'a> TryFrom<&'a [u8]> for AcpiGenlEvent {
    type Error = DecodeError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let raw: &RawAcpiGenlEvent = bytemuck::from_bytes(value);
        let devc_bytes = must_cast_ref::<_, [u8; 20]>(&raw.device_class);
        let devc = CStr::from_bytes_until_nul(devc_bytes)
            .map_err(|_| DecodeError::from("device_class is not null terminated"))?
            .to_str()
            .map_err(|_| DecodeError::from("device_class is not valid utf8"))?;

        Ok(Self {
            device_class: ArrayString::from_str(devc).expect("[BUG] device_clas.len() > 20?!"),
            kind: raw.kind,
            data: raw.data,
        })
    }
}

#[derive(Debug, Clone)]
pub struct AcpiGenl {
    pub cmd: AcpiGenlCmd,
    pub nlas: Vec<AcpiGenlAttr>,
}

impl GenlFamily for AcpiGenl {
    fn family_name() -> &'static str {
        "acpi_event"
    }

    fn version(&self) -> u8 {
        1
    }

    fn command(&self) -> u8 {
        self.cmd as u8
    }
}

impl ParseableParametrized<[u8], GenlHeader> for AcpiGenl {
    fn parse_with_param(buf: &[u8], header: GenlHeader) -> Result<Self, DecodeError> {
        Ok(Self {
            cmd: header.cmd.try_into()?,
            nlas: parse_nlas(buf)?,
        })
    }
}

fn parse_nlas(buf: &[u8]) -> Result<Vec<AcpiGenlAttr>, DecodeError> {
    NlasIterator::new(buf)
        .map(|nla| nla.and_then(|n| AcpiGenlAttr::parse(&n)))
        .collect::<Result<Vec<_>, _>>()
}
