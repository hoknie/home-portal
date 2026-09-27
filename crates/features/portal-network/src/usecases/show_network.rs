use std::sync::Arc;

use portal_config::{ConfigStore, Revisioned};
use portal_feature::ApiError;

use crate::services::configured_network;
use crate::types::NetworkSettings;

#[derive(Clone)]
pub struct ShowNetwork {
    configuration: Arc<ConfigStore>,
}

impl ShowNetwork {
    pub fn new(configuration: Arc<ConfigStore>) -> ShowNetwork {
        ShowNetwork { configuration }
    }

    pub fn run(&self) -> Result<Revisioned<NetworkSettings>, ApiError> {
        let snapshot = self.configuration.read();
        let settings = configured_network(&snapshot.document)?;
        Ok(Revisioned::new(settings, snapshot.revision))
    }
}
