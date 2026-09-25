use std::sync::Arc;

use portal_config::ConfigStore;
use portal_feature::{StatusChange, StatusObserver};

use super::Outbox;
use crate::types::{Outgoing, TelegramSection};

pub struct TelegramNotifier {
    configuration: Arc<ConfigStore>,
    outbox: Arc<Outbox>,
}

impl TelegramNotifier {
    pub fn new(configuration: Arc<ConfigStore>, outbox: Arc<Outbox>) -> TelegramNotifier {
        TelegramNotifier {
            configuration,
            outbox,
        }
    }

    pub fn message_of(change: &StatusChange) -> String {
        let mut text = format!("{}: {} → {}", change.name, change.was, change.now);
        if let Some(error) = &change.error {
            text.push_str(&format!("\n{error}"));
        }
        text
    }
}

impl StatusObserver for TelegramNotifier {
    fn changed(&self, change: &StatusChange) {
        if !change.notify {
            return;
        }
        let settings = match TelegramSection::read(&self.configuration.read().document) {
            Ok(settings) => settings,
            Err(problem) => {
                tracing::warn!(%problem, "the telegram settings could not be read");
                return;
            }
        };
        if !settings.announces(&change.now, change.from_unknown()) {
            return;
        }
        let Some(chat_id) = settings.chat_id.clone() else {
            return;
        };
        self.outbox.push(Outgoing {
            chat_id,
            text: Self::message_of(change),
        });
    }
}
