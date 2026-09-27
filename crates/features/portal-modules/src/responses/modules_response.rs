use serde::Serialize;

use super::ModuleResponse;
use crate::types::ModuleView;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ModulesResponse {
    pub modules: Vec<ModuleResponse>,
}

impl ModulesResponse {
    pub fn of(views: &[ModuleView]) -> ModulesResponse {
        ModulesResponse {
            modules: views.iter().map(ModuleResponse::of).collect(),
        }
    }
}
