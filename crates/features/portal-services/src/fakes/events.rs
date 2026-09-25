use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use portal_feature::{EventSink, PortalEvent};

use crate::ports::Publishing;
use crate::types::ServicesPorts;

#[derive(Default)]
pub struct Recorder {
    pub events: Mutex<Vec<PortalEvent>>,
}

#[async_trait]
impl EventSink for Recorder {
    fn emit(&self, event: PortalEvent) {
        self.events.lock().unwrap().push(event);
    }

    async fn settle(&self, _within: Duration) {}
}

pub fn quiet_ports(publishing: Arc<dyn Publishing>) -> ServicesPorts {
    ServicesPorts {
        observers: Vec::new(),
        publishing,
        events: Arc::new(Recorder::default()),
    }
}
