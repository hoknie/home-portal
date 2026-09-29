use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct MoveRequest {
    pub from: String,
    pub to: String,
}
