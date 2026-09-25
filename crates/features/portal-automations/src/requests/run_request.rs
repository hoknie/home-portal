use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RunRequest {
    #[serde(default)]
    pub script: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub timeout_seconds: Option<i64>,
}
