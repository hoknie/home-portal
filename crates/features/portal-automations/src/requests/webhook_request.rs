use serde::Deserialize;

use super::RunRequest;
use crate::types::{RawMarks, RawRun, RawWebhook};

#[derive(Debug, Clone, Deserialize)]
pub struct WebhookRequest {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub variables: Vec<String>,
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub run: Option<RunRequest>,
    #[serde(default)]
    pub with_token: bool,
}

impl WebhookRequest {
    pub fn into_raw(self, id: &str, token_sha256: Option<String>) -> RawWebhook {
        RawWebhook {
            id: id.to_string(),
            title: self.title,
            marks: RawMarks {
                enabled: self.enabled,
                tags: self.tags,
            },
            variables: self.variables,
            action: self.action,
            run: self.run.map(|run| RawRun {
                script: run.script,
                args: run.args,
                timeout_seconds: run.timeout_seconds,
            }),
            token_sha256,
        }
    }
}
