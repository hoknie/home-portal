use portal_model::ServiceState;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::types::Transition;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitionResponse {
    #[serde(with = "time::serde::rfc3339")]
    pub at: OffsetDateTime,
    pub from: ServiceState,
    pub to: ServiceState,
    pub error: Option<String>,
}

impl TransitionResponse {
    pub fn of(transition: &Transition) -> TransitionResponse {
        TransitionResponse {
            at: OffsetDateTime::from_unix_timestamp(transition.at)
                .unwrap_or(OffsetDateTime::UNIX_EPOCH),
            from: transition.from,
            to: transition.to,
            error: transition.error.clone(),
        }
    }
}
