use std::sync::Arc;

use portal_config::ConfigStore;
use portal_feature::FieldError;
use portal_model::Publication;

use crate::services::publication_problems;

#[derive(Clone)]
pub struct CheckPublication {
    configuration: Arc<ConfigStore>,
}

impl CheckPublication {
    pub fn new(configuration: Arc<ConfigStore>) -> CheckPublication {
        CheckPublication { configuration }
    }

    pub fn run(&self, publication: &Publication) -> Vec<FieldError> {
        publication_problems(&self.configuration.read().document, publication)
    }
}
