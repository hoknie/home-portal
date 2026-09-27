use std::sync::Arc;

use portal_config::ConfigStore;
use portal_feature::FieldError;
use portal_model::Environments;

use crate::services::read_environments;

#[derive(Clone)]
pub struct CurrentEnvironments {
    configuration: Arc<ConfigStore>,
}

impl CurrentEnvironments {
    pub fn new(configuration: Arc<ConfigStore>) -> CurrentEnvironments {
        CurrentEnvironments { configuration }
    }

    pub fn run(&self) -> Result<Environments, Vec<FieldError>> {
        read_environments(&self.configuration.read().document)
    }
}
