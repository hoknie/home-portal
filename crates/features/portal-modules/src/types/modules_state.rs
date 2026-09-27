use crate::usecases::{CurrentModules, SwitchModule};

#[derive(Clone)]
pub struct ModulesState {
    pub current: CurrentModules,
    pub switch: SwitchModule,
}
