use time::{Duration, OffsetDateTime};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawEvent {
    pub summary: String,
    pub start: OffsetDateTime,
    pub duration: Option<Duration>,
    pub all_day: bool,
    pub location: Option<String>,
    pub rule: Option<String>,
    pub excluded: Vec<OffsetDateTime>,
}
