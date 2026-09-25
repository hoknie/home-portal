use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::Repeats;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub summary: String,
    #[serde(with = "time::serde::rfc3339")]
    pub start: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub end: Option<OffsetDateTime>,
    pub all_day: bool,
    pub location: Option<String>,
    pub calendar: Option<String>,
    pub repeats: Repeats,
}
