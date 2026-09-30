use std::sync::Arc;

use portal_config::ConfigStore;

use crate::services::read_settings;
use crate::types::PermissionSettings;

#[derive(Clone)]
pub struct ReadPermissionSettings {
    configuration: Arc<ConfigStore>,
}

impl ReadPermissionSettings {
    pub fn new(configuration: Arc<ConfigStore>) -> ReadPermissionSettings {
        ReadPermissionSettings { configuration }
    }

    pub fn run(&self) -> PermissionSettings {
        read_settings(&self.configuration.read().document)
    }
}
