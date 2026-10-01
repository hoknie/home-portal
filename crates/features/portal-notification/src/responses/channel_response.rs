use portal_feature::{ChannelReadiness, PortalEvent};
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::Value;

use super::DeliveryResponse;
use crate::types::ChannelView;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct MissingResponse {
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct LastErrorResponse {
    pub at: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ChannelResponse {
    pub name: String,
    pub readiness: String,
    pub missing: Option<MissingResponse>,
    #[schemars(with = "serde_json::Map<String, Value>")]
    pub settings: Value,
    pub last_delivery: Option<DeliveryResponse>,
    pub last_error: Option<LastErrorResponse>,
    pub queued: usize,
    pub dropped: usize,
}

impl ChannelResponse {
    pub fn of(view: &ChannelView) -> ChannelResponse {
        ChannelResponse {
            name: view.name.clone(),
            readiness: view.readiness.name().to_string(),
            missing: match &view.readiness {
                ChannelReadiness::Missing { field, message } => Some(MissingResponse {
                    field: field.clone(),
                    message: message.clone(),
                }),
                _ => None,
            },
            settings: view.settings.clone(),
            last_delivery: view.last.as_ref().map(DeliveryResponse::of),
            last_error: view
                .last_error
                .as_ref()
                .map(|(at, message)| LastErrorResponse {
                    at: PortalEvent::timestamp(*at),
                    message: message.clone(),
                }),
            queued: view.queued,
            dropped: view.dropped,
        }
    }
}
