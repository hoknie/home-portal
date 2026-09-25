use serde::Deserialize;

use super::RawMarks;
use crate::types::RawRun;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RawWebhook {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(flatten)]
    pub marks: RawMarks,
    #[serde(default)]
    pub variables: Vec<String>,
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub run: Option<RawRun>,
    #[serde(default)]
    pub token_sha256: Option<String>,
}
