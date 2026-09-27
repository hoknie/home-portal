use std::sync::Arc;

use portal_feature::{EventName, EventSink, PortalEvent, Visitor};
use time::OffsetDateTime;

use crate::services::SessionGate;

#[derive(Clone)]
pub struct SignOut {
    gate: SessionGate,
    events: Arc<dyn EventSink>,
}

impl SignOut {
    pub fn new(gate: SessionGate, events: Arc<dyn EventSink>) -> SignOut {
        SignOut { gate, events }
    }

    pub fn run(&self, token: Option<&str>, visitor: Visitor) {
        let signed_in = token.and_then(|token| self.gate.principal_of(token));
        if let Some(token) = token {
            self.gate.sessions.end(token);
        }
        if let Some(principal) = signed_in {
            self.events.emit(PortalEvent::visited(
                EventName::UserSignedOut,
                &Visitor {
                    user: principal.name,
                    ..visitor
                },
                OffsetDateTime::now_utc(),
            ));
        }
    }
}
