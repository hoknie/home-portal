use serde::Serialize;

use crate::types::ModuleView;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ModuleResponse {
    pub name: &'static str,
    pub enabled: bool,
    pub requires: Vec<&'static str>,
    pub required_by: Vec<&'static str>,
}

impl ModuleResponse {
    pub fn of(view: &ModuleView) -> ModuleResponse {
        ModuleResponse {
            name: view.module.name(),
            enabled: view.enabled,
            requires: view.requires.iter().map(|module| module.name()).collect(),
            required_by: view
                .required_by
                .iter()
                .map(|module| module.name())
                .collect(),
        }
    }
}
