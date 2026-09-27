use portal_config::{Revision, Revisioned};
use portal_feature::ApiError;

use crate::repositories::set_token;
use crate::services::{Views, WebhookWriter};
use crate::types::WebhookView;

#[derive(Clone)]
pub struct RemoveToken {
    writer: WebhookWriter,
    views: Views,
}

impl RemoveToken {
    pub fn new(writer: WebhookWriter, views: Views) -> RemoveToken {
        RemoveToken { writer, views }
    }

    pub async fn run(
        &self,
        id: &str,
        revision: &Revision,
    ) -> Result<Revisioned<Vec<WebhookView>>, ApiError> {
        let snapshot = self
            .writer
            .write(id, revision, |document, index| {
                set_token(document, index, None)
            })
            .await?;
        Ok(Revisioned::new(
            self.views.webhooks(&snapshot.document),
            snapshot.revision,
        ))
    }
}
