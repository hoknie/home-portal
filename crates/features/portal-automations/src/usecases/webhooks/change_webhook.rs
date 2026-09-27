use std::sync::Arc;

use portal_config::{ConfigStore, Revision, Revisioned};
use portal_feature::ApiError;

use crate::repositories::replace_webhook;
use crate::services::{Views, WebhookWriter};
use crate::types::{AutomationsSection, RawWebhook, Webhook, WebhookView};

#[derive(Clone)]
pub struct ChangeWebhook {
    configuration: Arc<ConfigStore>,
    writer: WebhookWriter,
    views: Views,
}

impl ChangeWebhook {
    pub fn new(
        configuration: Arc<ConfigStore>,
        writer: WebhookWriter,
        views: Views,
    ) -> ChangeWebhook {
        ChangeWebhook {
            configuration,
            writer,
            views,
        }
    }

    pub async fn run(
        &self,
        id: &str,
        mut raw: RawWebhook,
        revision: &Revision,
    ) -> Result<Revisioned<WebhookView>, ApiError> {
        raw.token_sha256 = AutomationsSection::read(&self.configuration.read().document)
            .ok()
            .and_then(|section| section.webhooks.into_iter().find(|stored| stored.id == id))
            .ok_or(ApiError::NotFound(Webhook::UNKNOWN))?
            .token_sha256;
        let webhook = Webhook::decode(&raw).map_err(ApiError::Invalid)?;
        let snapshot = self
            .writer
            .write(id, revision, |document, index| {
                replace_webhook(document, index, &webhook)
            })
            .await?;
        Ok(Revisioned::new(
            self.views.webhook(webhook),
            snapshot.revision,
        ))
    }
}
