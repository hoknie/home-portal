use std::sync::Arc;

use portal_automations::{Choice, Directory};
use portal_config::ConfigStore;
use portal_model::Environment;
use portal_services::ServicesSection;

pub struct AutomationDirectory {
    pub configuration: Arc<ConfigStore>,
}

impl Directory for AutomationDirectory {
    fn services(&self) -> Vec<Choice> {
        ServicesSection::read(&self.configuration.read().document)
            .map(|section| {
                section
                    .services
                    .into_iter()
                    .map(|entry| Choice {
                        id: entry.id,
                        name: entry.name,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn users(&self) -> Vec<String> {
        portal_auth::user_names(&self.configuration.read().document)
    }

    fn environments(&self) -> Vec<String> {
        let mut names: Vec<String> =
            portal_network::read_environments(&self.configuration.read().document)
                .map(|environments| {
                    environments
                        .names()
                        .iter()
                        .map(|name| name.as_str().to_string())
                        .collect()
                })
                .unwrap_or_default();
        if !names.iter().any(|name| name == Environment::INTERNET) {
            names.push(Environment::INTERNET.to_string());
        }
        names
    }
}
