use axum::Router;
use axum::routing::get;
use portal_feature::Feature;

use crate::controllers::health;

pub struct HealthFeature;

impl HealthFeature {
    pub const NAME: &'static str = "health";
    pub const PATH: &'static str = "/health";
}

impl Feature for HealthFeature {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn router(&self) -> Router {
        Router::new()
    }

    fn public_router(&self) -> Router {
        Router::new().route(Self::PATH, get(health))
    }
}
