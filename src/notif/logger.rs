use std::convert::Infallible;

use super::{EvConsumer, Notification};

#[derive(Default, Clone, Copy)]
pub struct LoggerNotifier;

impl EvConsumer for LoggerNotifier {
    type Error = Infallible;

    async fn consume(&self, notif: Notification) -> Result<(), Infallible> {
        println!(
            "received notification event: {:?}, priority: {:?}",
            notif.event, notif.priority,
        );

        Ok(())
    }
}
