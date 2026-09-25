use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct QueuedResponse {
    pub run_id: String,
}
