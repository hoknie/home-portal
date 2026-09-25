use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IconState {
    pub service: String,
    pub source: String,
    pub available: bool,
    #[serde(with = "time::serde::rfc3339::option")]
    pub fetched_at: Option<OffsetDateTime>,
    pub problem: Option<String>,
}
