use portal_feature::Notification;

use crate::services::ChannelSet;
use crate::types::Delivery;

#[derive(Clone)]
pub struct SendNotification {
    channels: ChannelSet,
}

impl SendNotification {
    pub const NO_CHANNEL: &'static str = "no notification channel is enabled and configured";
    pub const MODULE_OFF: &'static str = "the notifications module is off";

    pub fn new(channels: ChannelSet) -> SendNotification {
        SendNotification { channels }
    }

    pub async fn run(
        &self,
        channel: Option<&str>,
        notification: &Notification,
    ) -> Result<Vec<Delivery>, String> {
        if !self.channels.module_on() {
            return Err(Self::MODULE_OFF.to_string());
        }
        let targets = match channel {
            Some(name) => {
                let entry = self
                    .channels
                    .find(name)
                    .ok_or_else(|| format!("there is no notification channel {name}"))?;
                let snapshot = self.channels.configuration.read();
                let readiness = entry
                    .channel
                    .readiness(&snapshot.document, self.channels.configuration.as_ref());
                if !readiness.is_ready() {
                    return Err(format!("the channel {name} is not configured"));
                }
                vec![entry]
            }
            None => self.channels.ready(),
        };
        if targets.is_empty() {
            return Err(Self::NO_CHANNEL.to_string());
        }
        let mut deliveries = Vec::new();
        for entry in targets {
            deliveries.push(self.channels.deliver(entry, notification).await);
        }
        Ok(deliveries)
    }
}
