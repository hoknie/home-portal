use std::sync::Arc;

use portal_config::{ConfigStore, Revisioned};
use portal_feature::ApiError;
use portal_model::Environment;

use crate::services::Showcase;
use crate::types::ShownService;

#[derive(Clone)]
pub struct ListServices {
    configuration: Arc<ConfigStore>,
    showcase: Showcase,
}

impl ListServices {
    pub fn new(configuration: Arc<ConfigStore>, showcase: Showcase) -> ListServices {
        ListServices {
            configuration,
            showcase,
        }
    }

    pub fn run(
        &self,
        environment: &Environment,
    ) -> Result<Revisioned<Vec<ShownService>>, ApiError> {
        let snapshot = self.configuration.read();
        let services = self.showcase.listed(&snapshot.document, environment)?;
        Ok(Revisioned::new(services, snapshot.revision))
    }
}
