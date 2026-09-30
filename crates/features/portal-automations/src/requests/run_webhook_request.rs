use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct RunWebhookRequest {
    #[serde(default)]
    pub variables: BTreeMap<String, String>,
}
