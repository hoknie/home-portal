use serde_json::Value;
use time::OffsetDateTime;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SyncState {
    pub reachable: bool,
    pub applied: Option<Value>,
    pub last_applied_at: Option<OffsetDateTime>,
    pub last_error: Option<String>,
}
