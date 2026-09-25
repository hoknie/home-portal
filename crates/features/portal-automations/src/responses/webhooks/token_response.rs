use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct TokenResponse {
    pub token: String,
}
