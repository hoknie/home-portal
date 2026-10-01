use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::{LatencyPointResponse, TransitionResponse, UptimeResponse};
use crate::types::HistoryView;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct HistoryResponse {
    pub range: String,
    #[serde(with = "time::serde::rfc3339")]
    #[schemars(with = "String")]
    pub from: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    #[schemars(with = "String")]
    pub to: OffsetDateTime,
    pub step_seconds: i64,
    pub uptime: Vec<UptimeResponse>,
    pub points: Vec<LatencyPointResponse>,
    pub transitions: Vec<TransitionResponse>,
}

impl HistoryResponse {
    pub fn of(view: &HistoryView) -> HistoryResponse {
        let time =
            |at: i64| OffsetDateTime::from_unix_timestamp(at).unwrap_or(OffsetDateTime::UNIX_EPOCH);
        HistoryResponse {
            range: view.range.name().to_string(),
            from: time(view.from),
            to: time(view.to),
            step_seconds: view.step,
            uptime: view.uptime.iter().map(UptimeResponse::of).collect(),
            points: view.points.iter().map(LatencyPointResponse::of).collect(),
            transitions: view
                .transitions
                .iter()
                .map(TransitionResponse::of)
                .collect(),
        }
    }
}
