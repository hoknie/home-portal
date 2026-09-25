use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use portal_config::ConfigStore;
use portal_feature::Feature;

use crate::controllers::list;

pub struct SecretsFeature {
    configuration: Arc<ConfigStore>,
}

impl SecretsFeature {
    pub const NAME: &'static str = "secrets";
    pub const PATH: &'static str = "/api/secrets";

    pub fn new(configuration: Arc<ConfigStore>) -> SecretsFeature {
        SecretsFeature { configuration }
    }
}

impl Feature for SecretsFeature {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn router(&self) -> Router {
        Router::new()
            .route(Self::PATH, get(list))
            .with_state(self.configuration.clone())
    }
}
