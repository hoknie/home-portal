use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use portal_config::ConfigStore;
use portal_feature::Feature;

use crate::controllers::list;
use crate::types::SecretsState;
use crate::usecases::ListSecrets;

pub struct SecretsFeature {
    state: SecretsState,
}

impl SecretsFeature {
    pub const NAME: &'static str = "secrets";
    pub const PATH: &'static str = "/api/secrets";

    pub fn new(configuration: Arc<ConfigStore>) -> SecretsFeature {
        SecretsFeature {
            state: SecretsState {
                list: ListSecrets::new(configuration),
            },
        }
    }
}

impl Feature for SecretsFeature {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn router(&self) -> Router {
        Router::new()
            .route(Self::PATH, get(list))
            .with_state(self.state.clone())
    }
}
