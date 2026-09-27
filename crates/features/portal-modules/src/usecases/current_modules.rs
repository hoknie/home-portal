use std::sync::Arc;

use portal_config::{ConfigStore, Revisioned};
use portal_feature::{ApiError, Module, ModuleSwitches};

use crate::types::ModuleView;

#[derive(Clone)]
pub struct CurrentModules {
    configuration: Arc<ConfigStore>,
}

impl CurrentModules {
    pub fn new(configuration: Arc<ConfigStore>) -> CurrentModules {
        CurrentModules { configuration }
    }

    pub fn run(&self) -> Result<Revisioned<Vec<ModuleView>>, ApiError> {
        let snapshot = self.configuration.read();
        let switches = ModuleSwitches::resolve(&snapshot.document).map_err(ApiError::Invalid)?;
        Ok(Revisioned::new(views(switches), snapshot.revision))
    }
}

pub fn views(switches: ModuleSwitches) -> Vec<ModuleView> {
    Module::ALL
        .into_iter()
        .map(|module| ModuleView {
            module,
            enabled: switches.is_on(module),
            requires: module.requires().to_vec(),
            required_by: switches.required_by(module),
        })
        .collect()
}
