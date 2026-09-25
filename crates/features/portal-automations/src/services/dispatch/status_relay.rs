use std::sync::Arc;

use portal_feature::{EventSink, PortalEvent, StatusChange, StatusObserver};
use time::OffsetDateTime;

use super::AutomationSink;

pub struct StatusRelay {
    pub sink: Arc<AutomationSink>,
}

impl StatusObserver for StatusRelay {
    fn changed(&self, change: &StatusChange) {
        self.sink.emit(PortalEvent::status_changed(
            change,
            OffsetDateTime::now_utc(),
        ));
    }
}
