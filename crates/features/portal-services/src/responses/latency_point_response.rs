use portal_model::ServiceState;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::types::LatencyPoint;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LatencyPointResponse {
    #[serde(with = "time::serde::rfc3339")]
    pub at: OffsetDateTime,
    pub state: ServiceState,
    pub average: Option<u32>,
    pub minimum: Option<u32>,
    pub maximum: Option<u32>,
}

impl LatencyPointResponse {
    pub fn of(point: &LatencyPoint) -> LatencyPointResponse {
        LatencyPointResponse {
            at: OffsetDateTime::from_unix_timestamp(point.at).unwrap_or(OffsetDateTime::UNIX_EPOCH),
            state: point.state,
            average: point.average,
            minimum: point.minimum,
            maximum: point.maximum,
        }
    }
}
