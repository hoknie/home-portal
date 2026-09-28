use portal_feature::{Notification, StatusChange, StatusObserver};

use super::ChannelSet;
use crate::types::Rules;

pub struct Notifier {
    pub channels: ChannelSet,
}

impl Notifier {
    pub fn message_of(change: &StatusChange) -> Notification {
        let mut text = format!("{} → {}", change.was, change.now);
        if let Some(error) = &change.error {
            text.push_str(&format!("\n{error}"));
        }
        Notification::new(change.name.clone(), text)
    }
}

impl StatusObserver for Notifier {
    fn changed(&self, change: &StatusChange) {
        if !change.notify || !self.channels.module_on() {
            return;
        }
        let rules = match Rules::read(&self.channels.configuration.read().document) {
            Ok(rules) => rules,
            Err(problem) => {
                tracing::warn!(%problem, "the notification rules could not be read");
                return;
            }
        };
        if !rules.announces(&change.now, change.from_unknown()) {
            return;
        }
        for entry in self.channels.ready() {
            entry
                .outbox
                .push(entry.channel.name(), Self::message_of(change));
        }
    }
}
