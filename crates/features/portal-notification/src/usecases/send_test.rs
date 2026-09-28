use portal_feature::{ApiError, Notification};

use crate::services::ChannelSet;
use crate::types::Delivery;

#[derive(Clone)]
pub struct SendTest {
    channels: ChannelSet,
}

impl SendTest {
    pub const TITLE: &'static str = "Home portal";
    pub const TEXT: &'static str =
        "This is a test notification. If you read it, the channel works.";
    pub const MODULE_OFF: &'static str = "the notifications module is off";

    pub fn new(channels: ChannelSet) -> SendTest {
        SendTest { channels }
    }

    pub async fn run(&self, name: &str) -> Result<Delivery, ApiError> {
        let entry = self
            .channels
            .find(name)
            .ok_or(ApiError::NotFound("no such notification channel"))?;
        if !self.channels.module_on() {
            return Err(ApiError::Conflict(Self::MODULE_OFF.to_string()));
        }
        let readiness = {
            let snapshot = self.channels.configuration.read();
            entry
                .channel
                .readiness(&snapshot.document, self.channels.configuration.as_ref())
        };
        if !readiness.is_ready() {
            return Err(ApiError::Conflict(format!(
                "the channel {name} is not configured"
            )));
        }
        Ok(self
            .channels
            .deliver(entry, &Notification::new(Self::TITLE, Self::TEXT))
            .await)
    }
}
