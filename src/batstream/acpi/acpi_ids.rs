use std::ops::{Deref, DerefMut};

use futures_channel::oneshot;
use futures_lite::StreamExt;
use netlink_packet_core::{
    NetlinkHeader, NetlinkMessage, NetlinkPayload, NLM_F_ACK, NLM_F_REQUEST,
};
use netlink_packet_generic::{
    constants::GENL_ID_CTRL,
    ctrl::{
        nlas::{GenlCtrlAttrs, McastGrpAttrs},
        GenlCtrl, GenlCtrlCmd,
    },
    GenlFamily, GenlHeader, GenlMessage,
};
use netlink_proto::{
    new_connection,
    sys::{protocols::NETLINK_GENERIC, SocketAddr},
    ConnectionHandle,
};

type Msg = GenlMessage<GenlCtrl>;

pub type Result<T> = core::result::Result<T, netlink_proto::Error<Msg>>;

struct ConnGuard {
    handle: ConnectionHandle<Msg>,
    cancel_tx: Option<oneshot::Sender<()>>,
}

impl Drop for ConnGuard {
    fn drop(&mut self) {
        self.cancel_tx
            .take()
            .unwrap()
            .send(())
            .expect("[BUG] nl connection died");
    }
}

impl Deref for ConnGuard {
    type Target = ConnectionHandle<Msg>;

    fn deref(&self) -> &ConnectionHandle<Msg> {
        &self.handle
    }
}

impl DerefMut for ConnGuard {
    fn deref_mut(&mut self) -> &mut ConnectionHandle<Msg> {
        &mut self.handle
    }
}

fn spawn_connection() -> Result<ConnGuard> {
    let (mut conn, handle, _) = new_connection(NETLINK_GENERIC)?;
    let (cancel_tx, mut cancel_rx) = oneshot::channel();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                biased;
                _ = (&mut cancel_rx) => break,
                _ = (&mut conn) => {},
            }
        }
    });

    Ok(ConnGuard {
        handle,
        cancel_tx: Some(cancel_tx),
    })
}

fn family_and_group_request() -> NetlinkMessage<Msg> {
    let ctrl = GenlCtrl {
        cmd: GenlCtrlCmd::GetFamily,
        nlas: vec![GenlCtrlAttrs::FamilyName("acpi_event".into())],
    };
    let header = GenlHeader {
        cmd: ctrl.command(),
        version: ctrl.version(),
    };
    let genl = GenlMessage::from_parts(header, ctrl);
    let mut nlheader = NetlinkHeader::default();
    nlheader.flags = NLM_F_REQUEST | NLM_F_ACK;
    nlheader.message_type = GENL_ID_CTRL;
    let mut msg = NetlinkMessage::new(nlheader, NetlinkPayload::InnerMessage(genl));
    msg.finalize();

    msg
}

fn find_acpi_group_id(attrs: Vec<Vec<McastGrpAttrs>>) -> Option<u32> {
    attrs
        .into_iter()
        .flat_map(|attrs| {
            let mut grp_id = None;
            for attr in attrs {
                match attr {
                    McastGrpAttrs::Name(name) if name != "acpi_mc_group" => return None,
                    McastGrpAttrs::Id(id) => grp_id = Some(id),
                    _ => {}
                }
            }
            grp_id
        })
        .next()
}

pub async fn get_family_and_group() -> Result<(u16, u32)> {
    let mut handle = spawn_connection()?;
    let req = family_and_group_request();
    let mut req_rx = handle.request(req, SocketAddr::new(0, 0))?;

    let mut family_id = None;
    let mut group_id = None;
    while let Some(msg) = req_rx.next().await {
        let NetlinkPayload::InnerMessage(GenlMessage { payload: ctrl, .. }) = msg.payload else {
            continue;
        };
        let mut attr_iter = ctrl.nlas.into_iter();
        while family_id.is_none() || group_id.is_none() {
            match attr_iter.next() {
                Some(GenlCtrlAttrs::FamilyId(id)) => family_id = Some(id),
                Some(GenlCtrlAttrs::McastGroups(grps)) => group_id = find_acpi_group_id(grps),
                None => break,
                _ => {}
            }
        }
    }

    Ok((
        family_id.expect("[BUG] unable to get family id"),
        group_id.expect("[BUG] unable to get group id"),
    ))
}
