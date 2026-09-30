use std::sync::Arc;

use axum::Router;
use axum::http::Method;
use axum::routing::{get, put};
use portal_config::ConfigStore;
use portal_feature::{
    Action, Area, Feature, ModulePreparer, ModuleSwitches, Right, Rule, Validator,
};

use crate::controllers::{show, switch};
use crate::types::ModulesState;
use crate::usecases::{CurrentModules, SwitchModule};

pub struct ModulesFeature {
    state: ModulesState,
}

impl ModulesFeature {
    pub const NAME: &'static str = "modules";
    pub const PATH: &'static str = "/api/modules";
    pub const ITEM: &'static str = "/api/modules/{name}";

    pub fn new(
        configuration: Arc<ConfigStore>,
        preparers: Vec<Arc<dyn ModulePreparer>>,
    ) -> ModulesFeature {
        ModulesFeature {
            state: ModulesState {
                current: CurrentModules::new(configuration.clone()),
                switch: SwitchModule::new(configuration, preparers),
            },
        }
    }
}

impl Feature for ModulesFeature {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn router(&self) -> Router {
        Router::new()
            .route(Self::PATH, get(show))
            .route(Self::ITEM, put(switch))
            .with_state(self.state.clone())
    }

    fn rules(&self) -> Vec<Rule> {
        vec![
            Rule::needs(
                Method::GET,
                Self::PATH,
                &[Right {
                    area: Area::Modules,
                    action: Action::Read,
                }],
            ),
            Rule::needs(
                Method::PUT,
                Self::ITEM,
                &[Right {
                    area: Area::Modules,
                    action: Action::Update,
                }],
            ),
        ]
    }

    fn validator(&self) -> Option<Validator> {
        Some(ModuleSwitches::errors)
    }
}
