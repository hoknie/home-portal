use std::collections::BTreeMap;
use std::sync::Arc;

use portal_config::{ConfigStore, Revision, Revisioned, Section};
use portal_feature::{ApiError, Principal};

use crate::repositories::{append_group, last_group_origin, last_origin};
use crate::services::{
    checked_group_name, checked_rights, groups_view, require_admin, require_editable, users_of,
};
use crate::types::GroupsView;

#[derive(Clone)]
pub struct CreateGroup {
    configuration: Arc<ConfigStore>,
}

impl CreateGroup {
    pub fn new(configuration: Arc<ConfigStore>) -> CreateGroup {
        CreateGroup { configuration }
    }

    pub async fn run(
        &self,
        actor: &Principal,
        name: &str,
        rights: &BTreeMap<String, Vec<String>>,
        revision: &Revision,
    ) -> Result<Revisioned<GroupsView>, ApiError> {
        require_admin(actor)?;
        let snapshot = self.configuration.read();
        require_editable(&snapshot.document)?;
        let section = users_of(&snapshot.document)?;
        let mut errors = Vec::new();
        let name = checked_group_name(name, &section, None)
            .map_err(|error| errors.push(error))
            .ok();
        let rights = checked_rights(rights)
            .map_err(|found| errors.extend(found))
            .ok();
        let (Some(name), Some(rights)) = (name, rights) else {
            return Err(ApiError::Invalid(errors));
        };
        let target = last_group_origin(&snapshot)
            .or_else(|| last_origin(&snapshot))
            .unwrap_or_else(|| self.configuration.home_of(Section::Users));
        let (_, written) = self
            .configuration
            .update(&target, revision, |document| {
                append_group(document, &name, &rights);
                Ok(())
            })
            .await?;
        Ok(Revisioned::new(
            groups_view(&written.document)?,
            written.revision,
        ))
    }
}
