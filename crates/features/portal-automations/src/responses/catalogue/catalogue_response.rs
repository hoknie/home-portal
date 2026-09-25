use portal_feature::{EventName, PortalEvent};
use serde::Serialize;

use super::{ChoiceResponse, ChoicesResponse, EventResponse, FieldResponse, WebhookChoiceResponse};
use crate::helpers::sample_of;
use crate::ports::Directory;
use crate::types::{Catalogue, FilterName, Webhook};

#[derive(Debug, Clone, Serialize)]
pub struct CatalogueResponse {
    pub events: Vec<EventResponse>,
    pub states: Vec<String>,
    pub choices: ChoicesResponse,
}

impl CatalogueResponse {
    pub fn of(
        directory: &dyn Directory,
        webhooks: &[Webhook],
        tags: Vec<String>,
    ) -> CatalogueResponse {
        let events = EventName::ALL
            .iter()
            .map(|event| EventResponse {
                name: event.name().to_string(),
                fields: Catalogue::fields_of(*event)
                    .into_iter()
                    .map(|field| FieldResponse {
                        name: field.to_string(),
                        sample: Self::sample(*event, field),
                    })
                    .collect(),
                filters: FilterName::for_event(*event)
                    .iter()
                    .map(|filter| filter.key().to_string())
                    .collect(),
            })
            .collect();
        CatalogueResponse {
            events,
            states: Catalogue::states()
                .into_iter()
                .map(str::to_string)
                .collect(),
            choices: ChoicesResponse {
                services: directory
                    .services()
                    .into_iter()
                    .map(|choice| ChoiceResponse {
                        id: choice.id,
                        name: choice.name,
                    })
                    .collect(),
                users: directory.users(),
                environments: directory.environments(),
                webhooks: webhooks
                    .iter()
                    .map(|webhook| WebhookChoiceResponse {
                        id: webhook.id.clone(),
                        name: webhook.title.clone(),
                        variables: webhook.variables.clone(),
                    })
                    .collect(),
                tags,
            },
        }
    }

    fn sample(event: EventName, field: &str) -> String {
        match field {
            PortalEvent::NAME_FIELD => event.name().to_string(),
            PortalEvent::AT_FIELD => "2026-01-01T03:00:00Z".to_string(),
            Catalogue::AUTOMATION_FIELD => "restart-media".to_string(),
            Catalogue::RUN_ID_FIELD => "42".to_string(),
            Catalogue::RUN_MANUAL_FIELD => "true".to_string(),
            Catalogue::RUN_BY_FIELD => "admin".to_string(),
            other => sample_of(other).to_string(),
        }
    }
}
