use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WidgetData {
    pub data: Value,
    #[serde(with = "time::serde::rfc3339")]
    pub fetched_at: OffsetDateTime,
    pub stale: bool,
    pub problem: Option<String>,
    pub refresh_seconds: u64,
    #[serde(default)]
    pub refreshing: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WidgetAnswer {
    Ready(WidgetData),
    Refreshing,
}

impl WidgetAnswer {
    pub fn into_ready(self) -> Option<WidgetData> {
        match self {
            WidgetAnswer::Ready(data) => Some(data),
            WidgetAnswer::Refreshing => None,
        }
    }

    pub fn map(self, change: impl FnOnce(WidgetData) -> WidgetData) -> WidgetAnswer {
        match self {
            WidgetAnswer::Ready(data) => WidgetAnswer::Ready(change(data)),
            WidgetAnswer::Refreshing => WidgetAnswer::Refreshing,
        }
    }
}
