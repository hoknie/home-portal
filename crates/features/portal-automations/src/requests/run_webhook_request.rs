use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct RunWebhookRequest {
    #[serde(default)]
    pub variables: BTreeMap<String, String>,
    #[serde(default)]
    pub body: Option<Value>,
}
