use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use portal_feature::Feature;

use crate::controllers::{portal, widget_data};
use crate::ports::{PublicLayout, PublicServices};
use crate::types::PublicState;

pub struct PublicFeature {
    state: PublicState,
}

impl PublicFeature {
    pub const NAME: &'static str = "public";
    pub const PORTAL_PATH: &'static str = "/api/public/portal";
    pub const WIDGET_PATH: &'static str = "/api/public/widgets/{id}/data";

    pub fn new(services: Arc<dyn PublicServices>, layout: Arc<dyn PublicLayout>) -> PublicFeature {
        PublicFeature {
            state: PublicState { services, layout },
        }
    }
}

impl Feature for PublicFeature {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn router(&self) -> Router {
        Router::new()
    }

    fn public_router(&self) -> Router {
        Router::new()
            .route(Self::PORTAL_PATH, get(portal))
            .route(Self::WIDGET_PATH, get(widget_data))
            .with_state(self.state.clone())
    }
}
