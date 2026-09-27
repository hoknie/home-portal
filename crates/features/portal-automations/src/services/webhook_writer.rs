use std::sync::Arc;

use portal_config::{ConfigStore, Revision, Snapshot};
use portal_feature::ApiError;
use toml_edit::DocumentMut;

use super::AutomationSink;
use crate::repositories::{webhook_origin, webhook_position};
use crate::types::Webhook;

#[derive(Clone)]
pub struct WebhookWriter {
    pub configuration: Arc<ConfigStore>,
    pub sink: Arc<AutomationSink>,
}

impl WebhookWriter {
    pub async fn write(
        &self,
        id: &str,
        revision: &Revision,
        change: impl FnOnce(&mut DocumentMut, usize),
    ) -> Result<Snapshot, ApiError> {
        let target = webhook_origin(&self.configuration.read(), id)
            .ok_or(ApiError::NotFound(Webhook::UNKNOWN))?;
        let (_, snapshot) = self
            .configuration
            .update(&target, revision, |document| {
                let index =
                    webhook_position(document, id).ok_or(ApiError::NotFound(Webhook::UNKNOWN))?;
                change(document, index);
                Ok(())
            })
            .await?;
        self.sink.cache.refresh(&snapshot.document);
        Ok(snapshot)
    }
}
