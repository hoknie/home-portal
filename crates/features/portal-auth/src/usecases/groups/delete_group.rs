use std::sync::Arc;

use portal_config::{ConfigStore, Revision, Revisioned};
use portal_feature::{ApiError, Principal};

use crate::repositories::{group_origin, group_position, remove_group};
use crate::services::{groups_view, require_admin, require_editable, users_of};
use crate::types::GroupsView;

pub const UNKNOWN_GROUP: &str = "no such group";
pub const BUILT_IN: &str = "admin is built in and cannot be changed or deleted";

#[derive(Clone)]
pub struct DeleteGroup {
    configuration: Arc<ConfigStore>,
}

impl DeleteGroup {
    pub fn new(configuration: Arc<ConfigStore>) -> DeleteGroup {
        DeleteGroup { configuration }
    }

    pub async fn run(
        &self,
        actor: &Principal,
        name: &str,
        revision: &Revision,
    ) -> Result<Revisioned<GroupsView>, ApiError> {
        require_admin(actor)?;
        if name == Principal::ADMIN {
            return Err(ApiError::Conflict(BUILT_IN.to_string()));
        }
        let snapshot = self.configuration.read();
        require_editable(&snapshot.document)?;
        let target = group_origin(&snapshot, name).ok_or(ApiError::NotFound(UNKNOWN_GROUP))?;
        let members: Vec<String> = users_of(&snapshot.document)?
            .users
            .into_iter()
            .filter(|user| user.group.as_deref() == Some(name))
            .map(|user| user.name)
            .collect();
        if !members.is_empty() {
            return Err(ApiError::Conflict(format!(
                "{name} still has members: {}; move them to another group first",
                members.join(", ")
            )));
        }
        let (_, written) = self
            .configuration
            .update(&target, revision, |document| {
                let index =
                    group_position(document, name).ok_or(ApiError::NotFound(UNKNOWN_GROUP))?;
                remove_group(document, index);
                Ok(())
            })
            .await?;
        Ok(Revisioned::new(
            groups_view(&written.document)?,
            written.revision,
        ))
    }
}
