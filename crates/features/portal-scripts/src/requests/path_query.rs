use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PathQuery {
    pub path: String,
}
