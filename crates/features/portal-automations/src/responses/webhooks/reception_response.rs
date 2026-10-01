use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ReceptionResponse {
    pub at: String,
    pub status: u16,
}
