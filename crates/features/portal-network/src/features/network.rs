use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use portal_config::ConfigStore;
use portal_feature::{Feature, Validator};

use crate::controllers::{change, show, show_environment};
use crate::services::{validate_environments, validate_network};
use crate::types::{EffectiveAddress, NetworkState};

fn validate_section(document: &toml_edit::DocumentMut) -> Vec<portal_feature::FieldError> {
    let mut errors = validate_network(document);
    errors.extend(validate_environments(document));
    errors
}

pub struct NetworkFeature {
    state: NetworkState,
}

impl NetworkFeature {
    pub const NAME: &'static str = "network";
    pub const PATH: &'static str = "/api/network";
    pub const ENVIRONMENT_PATH: &'static str = "/api/environment";

    pub fn new(configuration: Arc<ConfigStore>, effective: EffectiveAddress) -> NetworkFeature {
        NetworkFeature {
            state: NetworkState {
                configuration,
                effective,
            },
        }
    }
}

impl Feature for NetworkFeature {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn router(&self) -> Router {
        Router::new()
            .route(Self::PATH, get(show).put(change))
            .route(Self::ENVIRONMENT_PATH, get(show_environment))
            .with_state(self.state.clone())
    }

    fn validator(&self) -> Option<Validator> {
        Some(validate_section)
    }
}
