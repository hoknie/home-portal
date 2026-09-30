use std::sync::Arc;

use portal_config::{ConfigStore, Revision, Revisioned};
use portal_feature::{ApiError, Principal};

use crate::repositories::{origin, position, set_group};
use crate::services::{
    keeps_an_admin, may_give, may_touch, require_editable, unknown_group, users_of, users_view,
};
use crate::types::UsersView;

pub const UNKNOWN: &str = "no such user";

#[derive(Clone)]
pub struct ChangeUserGroup {
    configuration: Arc<ConfigStore>,
}

impl ChangeUserGroup {
    pub fn new(configuration: Arc<ConfigStore>) -> ChangeUserGroup {
        ChangeUserGroup { configuration }
    }

    pub async fn run(
        &self,
        actor: &Principal,
        name: &str,
        group: Option<String>,
        revision: &Revision,
    ) -> Result<Revisioned<UsersView>, ApiError> {
        let snapshot = self.configuration.read();
        require_editable(&snapshot.document)?;
        let target = origin(&snapshot, name).ok_or(ApiError::NotFound(UNKNOWN))?;
        let section = users_of(&snapshot.document)?;
        if let Some(error) = unknown_group(&section, group.as_deref()) {
            return Err(ApiError::Invalid(vec![error]));
        }
        may_touch(actor, &section, name)?;
        may_give(actor, &section, group.as_deref())?;
        keeps_an_admin(&section, name, group.as_deref())?;
        let (_, written) = self
            .configuration
            .update(&target, revision, |document| {
                let index = position(document, name).ok_or(ApiError::NotFound(UNKNOWN))?;
                set_group(document, index, group.as_deref());
                Ok(())
            })
            .await?;
        Ok(Revisioned::new(
            users_view(&written.document, &actor.name)?,
            written.revision,
        ))
    }
}
