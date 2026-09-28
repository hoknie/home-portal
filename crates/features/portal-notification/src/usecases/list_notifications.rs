use portal_config::{Revisioned, Snapshot};

use crate::services::ChannelSet;
use crate::types::{ChannelView, NotificationsView, Rules};

#[derive(Clone)]
pub struct ListNotifications {
    channels: ChannelSet,
}

impl ListNotifications {
    pub fn new(channels: ChannelSet) -> ListNotifications {
        ListNotifications { channels }
    }

    pub fn run(&self) -> Revisioned<NotificationsView> {
        let snapshot = self.channels.configuration.read();
        let view = self.view_of(&snapshot);
        Revisioned::new(view, snapshot.revision.clone())
    }

    pub fn view_of(&self, snapshot: &Snapshot) -> NotificationsView {
        let secrets = self.channels.configuration.as_ref();
        NotificationsView {
            enabled: self.channels.module_on(),
            rules: Rules::read(&snapshot.document).unwrap_or_default(),
            channels: self
                .channels
                .channels
                .iter()
                .map(|entry| {
                    let name = entry.channel.name();
                    ChannelView {
                        name: name.to_string(),
                        readiness: entry.channel.readiness(&snapshot.document, secrets),
                        settings: entry.channel.settings(&snapshot.document),
                        last: self.channels.book.last(name),
                        last_error: self.channels.book.last_error(name),
                        queued: entry.outbox.waiting(),
                        dropped: entry.outbox.dropped(),
                    }
                })
                .collect(),
        }
    }
}
