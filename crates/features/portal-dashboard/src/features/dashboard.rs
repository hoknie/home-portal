use std::sync::Arc;

use axum::Router;
use axum::http::Method;
use axum::routing::{get, put};
use portal_config::ConfigStore;
use portal_feature::{Action, Area, Feature, Right, Rule, Validator};

use crate::controllers::{
    create_library, delete_library, list_library, show, update, update_library,
};
use crate::services::validate_dashboard;
use crate::types::{DashboardState, NeedsOf};
use crate::usecases::{ChangeLayout, LibraryCases, ShowLayout};

const EDIT: &[Right] = &[Right::new(Area::Layout, Action::Update)];

pub struct DashboardFeature {
    state: DashboardState,
}

impl DashboardFeature {
    pub const NAME: &'static str = "dashboard";
    pub const PATH: &'static str = "/api/dashboard";
    pub const LIBRARY: &'static str = "/api/dashboard/library";
    pub const LIBRARY_ITEM: &'static str = "/api/dashboard/library/{id}";

    pub fn new(configuration: Arc<ConfigStore>) -> DashboardFeature {
        DashboardFeature {
            state: DashboardState {
                show: ShowLayout::new(configuration.clone()),
                change: ChangeLayout::new(configuration.clone()),
                library: LibraryCases::new(configuration),
            },
        }
    }

    pub fn with_needs(self, needs_of: NeedsOf) -> DashboardFeature {
        DashboardFeature {
            state: DashboardState {
                library: self.state.library.with_needs(needs_of),
                ..self.state
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
            .route(Self::LIBRARY, get(list_library).post(create_library))
            .route(
                Self::LIBRARY_ITEM,
                put(update_library).delete(delete_library),
            )
            .with_state(self.state.clone())
    }

    fn rules(&self) -> Vec<Rule> {
        vec![
            Rule::signed(Method::GET, Self::PATH),
            Rule::needs(Method::PUT, Self::PATH, EDIT),
            Rule::needs(Method::GET, Self::LIBRARY, EDIT),
            Rule::needs(Method::POST, Self::LIBRARY, EDIT),
            Rule::needs(Method::PUT, Self::LIBRARY_ITEM, EDIT),
            Rule::needs(Method::DELETE, Self::LIBRARY_ITEM, EDIT),
        ]
    }

    fn validator(&self) -> Option<Validator> {
        Some(validate_dashboard)
    }
}
