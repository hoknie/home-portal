use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use portal_config::{ConfigStore, Revision, Revisioned, Snapshot};
use portal_feature::{ApiError, Principal, Rights};

use super::delete_group::{BUILT_IN, UNKNOWN_GROUP};
use crate::repositories::{
    append_group, group_origin, group_position, member_origins, remove_group, rename_members,
    set_group_entry,
};
use crate::services::{
    checked_group_name, checked_rights, groups_view, require_admin, require_editable, users_of,
};
use crate::types::GroupsView;

#[derive(Clone)]
pub struct ChangeGroup {
    configuration: Arc<ConfigStore>,
}

impl ChangeGroup {
    pub fn new(configuration: Arc<ConfigStore>) -> ChangeGroup {
        ChangeGroup { configuration }
    }

    pub async fn run(
        &self,
        actor: &Principal,
        current: &str,
        name: &str,
        rights: &BTreeMap<String, Vec<String>>,
        revision: &Revision,
    ) -> Result<Revisioned<GroupsView>, ApiError> {
        require_admin(actor)?;
        if current == Principal::ADMIN {
            return Err(ApiError::Conflict(BUILT_IN.to_string()));
        }
        let snapshot = self.configuration.read();
        require_editable(&snapshot.document)?;
        let home = group_origin(&snapshot, current).ok_or(ApiError::NotFound(UNKNOWN_GROUP))?;
        let section = users_of(&snapshot.document)?;
        let mut errors = Vec::new();
        let name = checked_group_name(name, &section, Some(current))
            .map_err(|error| errors.push(error))
            .ok();
        let rights = checked_rights(rights)
            .map_err(|found| errors.extend(found))
            .ok();
        let (Some(name), Some(rights)) = (name, rights) else {
            return Err(ApiError::Invalid(errors));
        };
        let others: Vec<_> = member_origins(&snapshot, current)
            .into_iter()
            .filter(|path| path != &home)
            .collect();
        let written = if name == current || others.is_empty() {
            self.in_place(&home, current, &name, &rights, revision)
                .await?
        } else {
            self.across(&home, &others, current, &name, &rights, revision)
                .await?
        };
        Ok(Revisioned::new(
            groups_view(&written.document)?,
            written.revision,
        ))
    }

    async fn in_place(
        &self,
        home: &Path,
        current: &str,
        name: &str,
        rights: &Rights,
        revision: &Revision,
    ) -> Result<Snapshot, ApiError> {
        let (_, written) = self
            .configuration
            .update(home, revision, |document| {
                let index =
                    group_position(document, current).ok_or(ApiError::NotFound(UNKNOWN_GROUP))?;
                set_group_entry(document, index, name, rights);
                rename_members(document, current, name);
                Ok(())
            })
            .await?;
        Ok(written)
    }

    async fn across(
        &self,
        home: &Path,
        others: &[PathBuf],
        current: &str,
        name: &str,
        rights: &Rights,
        revision: &Revision,
    ) -> Result<Snapshot, ApiError> {
        let (_, mut written) = self
            .configuration
            .update(home, revision, |document| {
                append_group(document, name, rights);
                Ok(())
            })
            .await?;
        for path in others {
            let (_, next) = self
                .configuration
                .update(path, &written.revision, |document| {
                    rename_members(document, current, name);
                    Ok(())
                })
                .await?;
            written = next;
        }
        let (_, last) = self
            .configuration
            .update(home, &written.revision, |document| {
                rename_members(document, current, name);
                let index =
                    group_position(document, current).ok_or(ApiError::NotFound(UNKNOWN_GROUP))?;
                remove_group(document, index);
                Ok(())
            })
            .await?;
        Ok(last)
    }
}
