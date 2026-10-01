use schemars::JsonSchema;
use serde::Serialize;

use super::ChannelResponse;
use crate::types::NotificationsView;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct RulesResponse {
    pub states: Vec<String>,
    pub recovered: bool,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct NotificationsResponse {
    pub enabled: bool,
    pub rules: RulesResponse,
    pub channels: Vec<ChannelResponse>,
}

impl NotificationsResponse {
    pub fn of(view: &NotificationsView) -> NotificationsResponse {
        NotificationsResponse {
            enabled: view.enabled,
            rules: RulesResponse {
                states: view.rules.states.clone(),
                recovered: view.rules.recovered,
            },
            channels: view.channels.iter().map(ChannelResponse::of).collect(),
        }
    }
}
