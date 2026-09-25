use std::sync::Arc;
use std::time::Duration;

use portal_config::ConfigStore;
use portal_feature::{EventName, EventSink, PortalEvent};
use time::OffsetDateTime;

use crate::services::AutomationSink;

pub const WATCH_EVERY: Duration = Duration::from_secs(1);

pub async fn watch_forever(configuration: Arc<ConfigStore>, sink: Arc<AutomationSink>) {
    let mut seen = configuration.read().revision;
    loop {
        tokio::time::sleep(WATCH_EVERY).await;
        let snapshot = configuration.read();
        if snapshot.revision == seen {
            continue;
        }
        sink.cache.refresh(&snapshot.document);
        sink.emit(PortalEvent::of(
            EventName::ConfigurationChanged,
            OffsetDateTime::now_utc(),
            &[
                ("configuration.revision", snapshot.revision.as_str()),
                ("configuration.previous_revision", seen.as_str()),
            ],
        ));
        seen = snapshot.revision;
    }
}
