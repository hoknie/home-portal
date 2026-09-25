use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AcceptedResponse {
    pub accepted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
}
