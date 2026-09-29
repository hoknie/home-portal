use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct WriteRequest {
    pub content: String,
}
