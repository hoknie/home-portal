use std::sync::Arc;

use portal_config::ConfigStore;

use crate::types::UsersSection;

#[derive(Clone)]
pub struct UserNames {
    configuration: Arc<ConfigStore>,
}

impl UserNames {
    pub fn new(configuration: Arc<ConfigStore>) -> UserNames {
        UserNames { configuration }
    }

    pub fn run(&self) -> Vec<String> {
        UsersSection::read(&self.configuration.read().document)
            .map(|section| section.users.into_iter().map(|user| user.name).collect())
            .unwrap_or_default()
    }
}
