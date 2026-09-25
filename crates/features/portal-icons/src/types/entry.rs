use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub source: String,
    pub digest: String,
    pub content_type: String,
    #[serde(with = "time::serde::rfc3339")]
    pub fetched_at: OffsetDateTime,
}
