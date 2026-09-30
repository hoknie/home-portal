use std::sync::Arc;

use portal_config::ConfigStore;

use crate::types::{ServiceEntry, ServicesSection};

#[derive(Clone)]
pub struct ServiceEntries {
    configuration: Arc<ConfigStore>,
}

impl ServiceEntries {
    pub fn new(configuration: Arc<ConfigStore>) -> ServiceEntries {
        ServiceEntries { configuration }
    }

    pub fn run(&self) -> Result<Vec<ServiceEntry>, String> {
        self.configuration
            .read()
            .typed(ServicesSection::read)
            .map(|section| section.services.clone())
    }
}
