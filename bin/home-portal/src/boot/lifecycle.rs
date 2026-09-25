use std::net::SocketAddr;
use std::time::Duration;

use portal_feature::{EventName, PortalEvent};
use time::OffsetDateTime;

use crate::types::Registry;

pub const STOP_WAIT: Duration = Duration::from_secs(10);

pub fn started(registry: &Registry, address: SocketAddr) {
    registry.events.emit(PortalEvent::portal(
        EventName::PortalStarted,
        &address.to_string(),
        OffsetDateTime::now_utc(),
    ));
}

pub async fn stopping(registry: &Registry, address: SocketAddr) {
    registry.events.emit(PortalEvent::portal(
        EventName::PortalStopping,
        &address.to_string(),
        OffsetDateTime::now_utc(),
    ));
    registry.events.settle(STOP_WAIT).await;
}
