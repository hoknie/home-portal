use std::sync::Arc;

use portal_config::ConfigStore;
use portal_feature::{Channel, Module, ModuleSwitches, Notification};
use time::OffsetDateTime;

use super::{DeliveryBook, Outbox};
use crate::types::Delivery;

pub struct Registered {
    pub channel: Arc<dyn Channel>,
    pub outbox: Arc<Outbox>,
}

#[derive(Clone)]
pub struct ChannelSet {
    pub configuration: Arc<ConfigStore>,
    pub channels: Arc<Vec<Registered>>,
    pub book: Arc<DeliveryBook>,
}

impl ChannelSet {
    pub fn module_on(&self) -> bool {
        ModuleSwitches::resolve(&self.configuration.read().document)
            .map(|switches| switches.is_on(Module::Notifications))
            .unwrap_or(false)
    }

    pub fn find(&self, name: &str) -> Option<&Registered> {
        self.channels
            .iter()
            .find(|entry| entry.channel.name() == name)
    }

    pub fn ready(&self) -> Vec<&Registered> {
        let snapshot = self.configuration.read();
        self.channels
            .iter()
            .filter(|entry| {
                entry
                    .channel
                    .readiness(&snapshot.document, self.configuration.as_ref())
                    .is_ready()
            })
            .collect()
    }

    pub async fn deliver(&self, entry: &Registered, notification: &Notification) -> Delivery {
        let snapshot = self.configuration.read();
        let outcome = entry
            .channel
            .deliver(
                notification,
                &snapshot.document,
                self.configuration.as_ref(),
            )
            .await;
        let delivery = Delivery {
            channel: entry.channel.name().to_string(),
            at: OffsetDateTime::now_utc(),
            error: outcome.err(),
        };
        if let Some(problem) = &delivery.error {
            tracing::warn!(channel = %delivery.channel, %problem, "a notification was not delivered");
        }
        self.book.record(delivery.clone());
        delivery
    }
}
