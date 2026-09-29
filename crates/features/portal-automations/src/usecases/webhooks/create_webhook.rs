use std::sync::Arc;

use portal_config::{ConfigStore, Revision, Revisioned, Section};
use portal_feature::ApiError;

use crate::helpers::{new_token, new_webhook_id, token_hash};
use crate::repositories::append_webhook;
use crate::services::AutomationSink;
use crate::types::{CreatedWebhook, RawWebhook, Webhook, WebhookView};

#[derive(Clone)]
pub struct CreateWebhook {
    configuration: Arc<ConfigStore>,
    sink: Arc<AutomationSink>,
}

impl CreateWebhook {
    pub fn new(configuration: Arc<ConfigStore>, sink: Arc<AutomationSink>) -> CreateWebhook {
        CreateWebhook {
            configuration,
            sink,
        }
    }

    pub async fn run(
        &self,
        mut raw: RawWebhook,
        with_token: bool,
        revision: &Revision,
    ) -> Result<Revisioned<CreatedWebhook>, ApiError> {
        let token = with_token.then(new_token);
        raw.id = new_webhook_id();
        raw.token_sha256 = token.as_deref().map(token_hash);
        let webhook = Webhook::decode(&raw).map_err(ApiError::Invalid)?;
        let target = self.configuration.home_of(Section::Webhooks);
        let (_, snapshot) = self
            .configuration
            .update(&target, revision, |document| {
                append_webhook(document, &webhook);
                Ok(())
            })
            .await?;
        self.sink.cache.refresh(&snapshot.document);
        let created = CreatedWebhook {
            view: WebhookView {
                webhook,
                last: None,
            },
            token,
        };
        Ok(Revisioned::new(created, snapshot.revision))
    }
}
