use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use portal_config::ConfigStore;
use portal_feature::{Feature, Validator};

use crate::controllers::{show, update};
use crate::services::validate_dashboard;

pub struct DashboardFeature {
    configuration: Arc<ConfigStore>,
}

impl DashboardFeature {
    pub const NAME: &'static str = "dashboard";
    pub const PATH: &'static str = "/api/dashboard";

    pub fn new(configuration: Arc<ConfigStore>) -> DashboardFeature {
        DashboardFeature { configuration }
    }
}

impl Feature for DashboardFeature {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn router(&self) -> Router {
        Router::new()
            .route(Self::PATH, get(show).put(update))
            .with_state(self.configuration.clone())
    }

    fn validator(&self) -> Option<Validator> {
        Some(validate_dashboard)
    }
}
