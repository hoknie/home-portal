use portal_config::{Revision, Revisioned};
use portal_feature::ApiError;

use crate::repositories::remove_webhook;
use crate::services::{Views, WebhookWriter};
use crate::types::WebhookView;

#[derive(Clone)]
pub struct DeleteWebhook {
    writer: WebhookWriter,
    views: Views,
}

impl DeleteWebhook {
    pub fn new(writer: WebhookWriter, views: Views) -> DeleteWebhook {
        DeleteWebhook { writer, views }
    }

    pub async fn run(
        &self,
        id: &str,
        revision: &Revision,
    ) -> Result<Revisioned<Vec<WebhookView>>, ApiError> {
        let snapshot = self.writer.write(id, revision, remove_webhook).await?;
        Ok(Revisioned::new(
            self.views.webhooks(&snapshot.document),
            snapshot.revision,
        ))
    }
}
