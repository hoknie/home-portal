use std::sync::Arc;

use portal_config::ConfigStore;

use crate::services::scripts_settings;

#[derive(Clone)]
pub struct ScriptEditing {
    configuration: Arc<ConfigStore>,
}

impl ScriptEditing {
    pub fn new(configuration: Arc<ConfigStore>) -> ScriptEditing {
        ScriptEditing { configuration }
    }

    pub fn run(&self) -> bool {
        scripts_settings(&self.configuration.read().document).editing
    }
}
