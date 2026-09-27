use std::sync::Arc;

use portal_config::ConfigStore;

use crate::services::{read_environments, read_network};
use crate::types::NetworkReading;

#[derive(Clone)]
pub struct CurrentNetwork {
    configuration: Arc<ConfigStore>,
}

impl CurrentNetwork {
    pub fn new(configuration: Arc<ConfigStore>) -> CurrentNetwork {
        CurrentNetwork { configuration }
    }

    pub fn run(&self) -> NetworkReading {
        let snapshot = self.configuration.read();
        NetworkReading {
            settings: read_network(&snapshot.document),
            environments: read_environments(&snapshot.document),
        }
    }
}
