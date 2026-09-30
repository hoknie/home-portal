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
            settings: snapshot
                .typed(read_network)
                .map(|settings| (*settings).clone()),
            environments: snapshot
                .typed(read_environments)
                .map(|environments| (*environments).clone()),
        }
    }
}
