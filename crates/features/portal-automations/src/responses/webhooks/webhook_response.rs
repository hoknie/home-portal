use portal_feature::PortalEvent;
use serde::Serialize;

use super::{ReceptionResponse, WebhookActionResponse};
use crate::responses::MarksResponse;
use crate::types::{Reception, Webhook};

#[derive(Debug, Clone, Serialize)]
pub struct WebhookResponse {
    pub id: String,
    pub title: String,
    #[serde(flatten)]
    pub marks: MarksResponse,
    pub address: String,
    pub protected: bool,
    #[serde(flatten)]
    pub action: WebhookActionResponse,
    pub last_received: Option<ReceptionResponse>,
}

impl WebhookResponse {
    pub fn of(webhook: &Webhook, reception: Option<Reception>) -> WebhookResponse {
        WebhookResponse {
            id: webhook.id.clone(),
            title: webhook.title.clone(),
            marks: MarksResponse {
                enabled: webhook.enabled,
                tags: webhook.tags.clone(),
            },
            address: webhook.address(),
            protected: webhook.token_sha256.is_some(),
            action: WebhookActionResponse::of(webhook),
            last_received: reception.map(|reception| ReceptionResponse {
                at: PortalEvent::timestamp(reception.at),
                status: reception.status,
            }),
        }
    }
}
