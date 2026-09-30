use std::sync::Arc;

use portal_config::{ConfigStore, Revision, Revisioned};
use portal_feature::{ApiError, Principal};

use crate::repositories::{origin, position, remove};
use crate::services::{keeps_an_admin, may_touch, require_editable, users_of, users_view};
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
        actor: &Principal,
        name: &str,
        revision: &Revision,
    ) -> Result<Revisioned<UsersView>, ApiError> {
        let snapshot = self.configuration.read();
        require_editable(&snapshot.document)?;
        let target = origin(&snapshot, name).ok_or(ApiError::NotFound(UNKNOWN))?;
        if name == actor.name {
            return Err(ApiError::Conflict(YOURSELF.to_string()));
        }
        let section = users_of(&snapshot.document)?;
        if section.users.len() <= 1 {
            return Err(ApiError::Conflict(LAST.to_string()));
        }
        may_touch(actor, &section, name)?;
        keeps_an_admin(&section, name, None)?;
        let (_, written) = self
            .configuration
            .update(&target, revision, |document| {
                let index = position(document, name).ok_or(ApiError::NotFound(UNKNOWN))?;
                remove(document, index);
                Ok(())
            })
            .await?;
        Ok(Revisioned::new(
            users_view(&written.document, &actor.name)?,
            written.revision,
        ))
    }
}
