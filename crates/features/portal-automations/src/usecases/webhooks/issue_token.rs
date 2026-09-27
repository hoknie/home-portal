use portal_config::{Revision, Revisioned};
use portal_feature::ApiError;

use crate::helpers::{new_token, token_hash};
use crate::repositories::set_token;
use crate::services::WebhookWriter;

#[derive(Clone)]
pub struct IssueToken {
    writer: WebhookWriter,
}

impl IssueToken {
    pub fn new(writer: WebhookWriter) -> IssueToken {
        IssueToken { writer }
    }

    pub async fn run(&self, id: &str, revision: &Revision) -> Result<Revisioned<String>, ApiError> {
        let token = new_token();
        let hash = token_hash(&token);
        let snapshot = self
            .writer
            .write(id, revision, |document, index| {
                set_token(document, index, Some(&hash))
            })
            .await?;
        Ok(Revisioned::new(token, snapshot.revision))
    }
}
