//! ACPI events battery stream backed by netlink

use std::{
    iter::FilterMap,
    path::Path,
    pin::Pin,
    task::{Context, Poll},
    vec,
};

use futures_lite::{ready, Stream, StreamExt};
use netlink_packet_core::{NetlinkMessage, NetlinkPayload};
use netlink_packet_generic::GenlMessage;
use netlink_proto::{
    sys::{protocols::NETLINK_GENERIC, AsyncSocket, SocketAddr, TokioSocket},
    NetlinkCodec, NetlinkFramed,
};
use tokio_udev::Device;

use self::{
    acpi_event::{AcpiGenl, AcpiGenlAttr, AcpiGenlEvent},
    acpi_ids::get_family_and_group,
};

use super::{sysfs::BAT_BASE_PATH, udev_bat::extract_battery_cap, AdapterStatus, BatEvent};
mod acpi_event;
mod acpi_ids;

const fn group_bitmap(group: u32) -> u32 {
    if group != 0 {
        1 << (group - 1)
    } else {
        0
    }
}

type Msg = GenlMessage<AcpiGenl>;
type EvBuf = FilterMap<vec::IntoIter<AcpiGenlAttr>, fn(AcpiGenlAttr) -> Option<AcpiGenlEvent>>;

pub struct AcpiStream {
    family_id: u16,
    netlink: NetlinkFramed<Msg, TokioSocket, NetlinkCodec>,
    battery: Device,
    buf: Option<EvBuf>,
}

impl AcpiStream {
    pub async fn new(battery_device: &str) -> Self {
        let battery_path = Path::new(BAT_BASE_PATH).join(battery_device);
        let (family_id, group_id) = get_family_and_group()
            .await
            .expect("acpi_event family not found");
        let mut socket =
            TokioSocket::new(NETLINK_GENERIC).expect("failed to create netlink socket");
        let inner_socket = socket.socket_mut();
        let addr = SocketAddr::new(0, group_bitmap(group_id));
        inner_socket.bind(&addr).unwrap();
        inner_socket.set_non_blocking(true).unwrap();

        Self {
            family_id,
            netlink: NetlinkFramed::new(socket),
            battery: Device::from_syspath(&battery_path).expect("failed to get battery device"),
            buf: None,
        }
    }

    fn next_buf(&mut self) -> Option<BatEvent> {
        while let Some(ev) = self.buf.as_mut()?.next() {
            match ev.device_class() {
                "ac_adapter" => {
                    return if ev.data == 1 {
                        Some(BatEvent::Adapter(AdapterStatus::Connected))
                    } else {
                        Some(BatEvent::Adapter(AdapterStatus::Disconnected))
                    }
                }
                "battery" => return Some(BatEvent::Battery(extract_battery_cap(&self.battery))),
                _ => continue,
            }
        }
        self.buf = None;
        None
    }
}

impl Stream for AcpiStream {
    type Item = std::io::Result<BatEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(ev) = self.next_buf() {
            return Poll::Ready(Some(Ok(ev)));
        }

        loop {
            let genl_msg = match ready!(self.netlink.poll_next(cx)) {
                Some((
                    NetlinkMessage {
                        payload: NetlinkPayload::InnerMessage(genl_msg),
                        ..
                    },
                    _,
                )) if genl_msg.resolved_family_id() == self.family_id => genl_msg,
                None => return Poll::Ready(None),
                _ => continue,
            };
            let GenlMessage {
                payload: AcpiGenl { nlas, .. },
                ..
            } = genl_msg;
            let filter: fn(AcpiGenlAttr) -> Option<AcpiGenlEvent> = |nla| match nla {
                AcpiGenlAttr::Event(event) => Some(event),
                _ => None,
            };
            let buf = nlas.into_iter().filter_map(filter);
            self.buf = Some(buf);
            let Some(ev) = self.next_buf() else {
                continue;
            }; 
            return Poll::Ready(Some(Ok(ev)));
        }
    }
}
