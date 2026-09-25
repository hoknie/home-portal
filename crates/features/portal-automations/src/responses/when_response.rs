use serde::Serialize;

use super::StatesResponse;
use crate::types::Trigger;

#[derive(Debug, Clone, Serialize)]
pub struct WhenResponse {
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cron: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub services: Vec<String>,
    #[serde(flatten)]
    pub states: StatesResponse,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub users: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub environments: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub webhooks: Vec<String>,
}

impl WhenResponse {
    pub fn of(trigger: &Trigger) -> WhenResponse {
        let filters = trigger.filters.clone();
        WhenResponse {
            event: trigger.event.name().to_string(),
            cron: filters.cron.map(|cron| cron.expression),
            services: filters.services,
            states: StatesResponse::of(&filters.states),
            users: filters.users,
            environments: filters.environments,
            webhooks: filters.webhooks,
        }
    }
}
