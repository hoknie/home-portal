use std::sync::Arc;

use portal_config::{ConfigStore, Revisioned};
use portal_feature::ApiError;

use crate::services::groups_view;
use crate::types::GroupsView;

#[derive(Clone)]
pub struct ListGroups {
    configuration: Arc<ConfigStore>,
}

impl ListGroups {
    pub fn new(configuration: Arc<ConfigStore>) -> ListGroups {
        ListGroups { configuration }
    }

    pub fn run(&self) -> Result<Revisioned<GroupsView>, ApiError> {
        let snapshot = self.configuration.read();
        Ok(Revisioned::new(
            groups_view(&snapshot.document)?,
            snapshot.revision,
        ))
    }
}
