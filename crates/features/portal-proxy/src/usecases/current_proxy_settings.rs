use std::sync::Arc;

use portal_config::ConfigStore;
use portal_feature::FieldError;

use crate::services::read_settings;
use crate::types::ProxySettings;

#[derive(Clone)]
pub struct CurrentProxySettings {
    configuration: Arc<ConfigStore>,
}

impl CurrentProxySettings {
    pub fn new(configuration: Arc<ConfigStore>) -> CurrentProxySettings {
        CurrentProxySettings { configuration }
    }

    pub fn run(&self) -> Result<ProxySettings, Vec<FieldError>> {
        read_settings(&self.configuration.read().document)
    }
}
