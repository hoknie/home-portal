use std::sync::Arc;

use portal_config::{ConfigStore, Revision, Revisioned};
use portal_feature::ApiError;

use crate::repositories::{origin, position, remove};
use crate::services::{require_editable, users_of, users_view};
use crate::types::UsersView;

pub const UNKNOWN: &str = "no such user";
pub const YOURSELF: &str = "you cannot delete yourself";
pub const LAST: &str = "at least one user is required";

#[derive(Clone)]
pub struct DeleteUser {
    configuration: Arc<ConfigStore>,
}

impl DeleteUser {
    pub fn new(configuration: Arc<ConfigStore>) -> DeleteUser {
        DeleteUser { configuration }
    }

    pub async fn run(
        &self,
        you: &str,
        name: &str,
        revision: &Revision,
    ) -> Result<Revisioned<UsersView>, ApiError> {
        let snapshot = self.configuration.read();
        require_editable(&snapshot.document)?;
        let target = origin(&snapshot, name).ok_or(ApiError::NotFound(UNKNOWN))?;
        if name == you {
            return Err(ApiError::Conflict(YOURSELF.to_string()));
        }
        if users_of(&snapshot.document)?.users.len() <= 1 {
            return Err(ApiError::Conflict(LAST.to_string()));
        }
        let (_, written) = self
            .configuration
            .update(&target, revision, |document| {
                let index = position(document, name).ok_or(ApiError::NotFound(UNKNOWN))?;
                remove(document, index);
                Ok(())
            })
            .await?;
        Ok(Revisioned::new(
            users_view(&written.document, you)?,
            written.revision,
        ))
    }
}
