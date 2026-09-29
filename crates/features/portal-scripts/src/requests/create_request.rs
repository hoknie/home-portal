use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateRequest {
    pub path: String,
    pub content: String,
}
