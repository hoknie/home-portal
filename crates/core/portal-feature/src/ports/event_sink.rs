use std::time::Duration;

use async_trait::async_trait;

use crate::types::PortalEvent;

#[async_trait]
pub trait EventSink: Send + Sync {
    fn emit(&self, event: PortalEvent);

    async fn settle(&self, within: Duration);
}
