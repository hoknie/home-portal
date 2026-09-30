use std::sync::Arc;

use axum::Router;
use axum::http::Method;
use axum::routing::get;
use portal_config::ConfigStore;
use portal_feature::{Action, Area, Feature, Right, Rule, Validator};

use crate::controllers::{show, update};
use crate::services::validate_dashboard;
use crate::types::DashboardState;
use crate::usecases::{ChangeLayout, ShowLayout};

pub struct DashboardFeature {
    state: DashboardState,
}

impl DashboardFeature {
    pub const NAME: &'static str = "dashboard";
    pub const PATH: &'static str = "/api/dashboard";

    pub fn new(configuration: Arc<ConfigStore>) -> DashboardFeature {
        DashboardFeature {
            state: DashboardState {
                show: ShowLayout::new(configuration.clone()),
                change: ChangeLayout::new(configuration),
            },
        }
    }
}

impl Feature for DashboardFeature {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn router(&self) -> Router {
        Router::new()
            .route(Self::PATH, get(show).put(update))
            .with_state(self.state.clone())
    }

    fn rules(&self) -> Vec<Rule> {
        vec![
            Rule::signed(Method::GET, Self::PATH),
            Rule::needs(
                Method::PUT,
                Self::PATH,
                &[Right {
                    area: Area::Layout,
                    action: Action::Update,
                }],
            ),
        ]
    }

    fn validator(&self) -> Option<Validator> {
        Some(validate_dashboard)
    }
}
