use std::sync::Arc;

use portal_config::{ConfigStore, Revisioned};
use portal_feature::ApiError;

use crate::services::users_view;
use crate::types::UsersView;

#[derive(Clone)]
pub struct ListUsers {
    configuration: Arc<ConfigStore>,
}

impl ListUsers {
    pub fn new(configuration: Arc<ConfigStore>) -> ListUsers {
        ListUsers { configuration }
    }

    pub fn run(&self, you: &str) -> Result<Revisioned<UsersView>, ApiError> {
        let snapshot = self.configuration.read();
        Ok(Revisioned::new(
            users_view(&snapshot.document, you)?,
            snapshot.revision,
        ))
    }
}
