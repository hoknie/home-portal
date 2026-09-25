use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use portal_config::ConfigStore;
use portal_feature::{Feature, Loop, Validator};

use crate::controllers::{preview, serve_icon, serve_public_icon, state};
use crate::loops::refresh_forever;
use crate::services::{Icons, validate_icons};

pub struct IconsFeature {
    icons: Arc<Icons>,
}

impl IconsFeature {
    pub const NAME: &'static str = "icons";
    pub const PATH: &'static str = "/api/icons/{service}";
    pub const STATE_PATH: &'static str = "/api/icons";
    pub const PREVIEW_PATH: &'static str = "/api/icon-preview";
    pub const PUBLIC_PATH: &'static str = "/api/public/icons/{service}";

    pub fn new(configuration: Arc<ConfigStore>, catalog: &str) -> Result<IconsFeature, String> {
        Ok(IconsFeature {
            icons: Arc::new(Icons::new(configuration, catalog)?),
        })
    }

    pub fn icons(&self) -> Arc<Icons> {
        self.icons.clone()
    }
}

impl Feature for IconsFeature {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn router(&self) -> Router {
        Router::new()
            .route(Self::PATH, get(serve_icon))
            .route(Self::STATE_PATH, get(state))
            .route(Self::PREVIEW_PATH, post(preview))
            .with_state(self.icons.clone())
    }

    fn public_router(&self) -> Router {
        Router::new()
            .route(Self::PUBLIC_PATH, get(serve_public_icon))
            .with_state(self.icons.clone())
    }

    fn validator(&self) -> Option<Validator> {
        Some(validate_icons)
    }

    fn loops(&self) -> Vec<Loop> {
        vec![Box::pin(refresh_forever(self.icons.clone()))]
    }
}
