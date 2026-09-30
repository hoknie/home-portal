use std::sync::Arc;

use portal_config::ConfigStore;
use portal_feature::FieldError;
use portal_model::Language;

use crate::services::read_interface;

#[derive(Clone)]
pub struct CurrentInterface {
    configuration: Arc<ConfigStore>,
}

impl CurrentInterface {
    pub fn new(configuration: Arc<ConfigStore>) -> CurrentInterface {
        CurrentInterface { configuration }
    }

    pub fn run(&self) -> Result<Language, Vec<FieldError>> {
        self.configuration
            .read()
            .typed(read_interface)
            .map(|language| *language)
    }
}
