use serde_json::Value;
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct Cached {
    pub data: Value,
    pub fetched_at: OffsetDateTime,
    pub problem: Option<String>,
}
