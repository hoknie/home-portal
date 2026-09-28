use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post, put};
use portal_config::ConfigStore;
use portal_feature::{Channel, Feature, Loop, StatusObserver, Validator};

use crate::controllers::{change_channel, change_rules, list as list_route, send_test};
use crate::loops::send_forever;
use crate::services::{ChannelSet, DeliveryBook, Notifier, Outbox, Registered};
use crate::types::{NotificationsState, Rules};
use crate::usecases::{ChangeChannel, ChangeRules, ListNotifications, SendNotification, SendTest};

pub struct NotificationFeature {
    channels: ChannelSet,
}

impl NotificationFeature {
    pub const NAME: &'static str = "notification";
    pub const COLLECTION: &'static str = "/api/notifications";
    pub const CHANNEL: &'static str = "/api/notifications/channels/{name}";
    pub const TEST: &'static str = "/api/notifications/test";

    pub fn new(
        configuration: Arc<ConfigStore>,
        channels: Vec<Arc<dyn Channel>>,
    ) -> Result<NotificationFeature, String> {
        let snapshot = configuration.read();
        if let Some(problem) = Rules::problems(&snapshot.document).first() {
            return Err(format!("{}: {}", problem.field, problem.message));
        }
        for channel in &channels {
            if let Some(problem) = channel
                .problems(&snapshot.document, configuration.as_ref())
                .first()
            {
                return Err(format!("{}: {}", problem.field, problem.message));
            }
        }
        let registered = channels
            .into_iter()
            .map(|channel| Registered {
                channel,
                outbox: Arc::new(Outbox::default()),
            })
            .collect();
        Ok(NotificationFeature {
            channels: ChannelSet {
                configuration: configuration.clone(),
                channels: Arc::new(registered),
                book: Arc::new(DeliveryBook::default()),
            },
        })
    }

    pub fn observer(&self) -> Arc<dyn StatusObserver> {
        Arc::new(Notifier {
            channels: self.channels.clone(),
        })
    }

    pub fn send_notification(&self) -> SendNotification {
        SendNotification::new(self.channels.clone())
    }

    pub fn outbox(&self, channel: &str) -> Option<Arc<Outbox>> {
        self.channels
            .find(channel)
            .map(|entry| entry.outbox.clone())
    }
}

impl Feature for NotificationFeature {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn router(&self) -> Router {
        let list = ListNotifications::new(self.channels.clone());
        let state = NotificationsState {
            change_rules: ChangeRules::new(self.channels.clone(), list.clone()),
            change_channel: ChangeChannel::new(self.channels.clone(), list.clone()),
            send_test: SendTest::new(self.channels.clone()),
            list,
        };
        Router::new()
            .route(Self::COLLECTION, get(list_route).put(change_rules))
            .route(Self::CHANNEL, put(change_channel))
            .route(Self::TEST, post(send_test))
            .with_state(state)
    }

    fn validator(&self) -> Option<Validator> {
        Some(Rules::problems)
    }

    fn loops(&self) -> Vec<Loop> {
        (0..self.channels.channels.len())
            .map(|index| -> Loop { Box::pin(send_forever(self.channels.clone(), index)) })
            .collect()
    }
}
