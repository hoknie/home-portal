use std::sync::Arc;

use portal_config::ConfigStore;
use portal_model::ServiceStatus as Status;

use crate::services::Showcase;
use crate::types::ServicesSection;

#[derive(Clone)]
pub struct CurrentStatus {
    configuration: Arc<ConfigStore>,
    showcase: Showcase,
}

impl CurrentStatus {
    pub fn new(configuration: Arc<ConfigStore>, showcase: Showcase) -> CurrentStatus {
        CurrentStatus {
            configuration,
            showcase,
        }
    }

    pub fn run(&self, id: &str) -> Result<Status, String> {
        let known = ServicesSection::read(&self.configuration.read().document)?
            .services
            .iter()
            .any(|entry| entry.id == id);
        if !known {
            return Err(format!("no service {id}"));
        }
        Ok(self.showcase.board.status(id))
    }
}
