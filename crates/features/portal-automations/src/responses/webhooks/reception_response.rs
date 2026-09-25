use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ReceptionResponse {
    pub at: String,
    pub status: u16,
}
