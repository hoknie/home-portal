use std::sync::Arc;

use axum::Router;
use axum::http::Method;
use axum::routing::{get, post};
use portal_config::ConfigStore;
use portal_feature::{Action, Area, Feature, Loop, Right, Rule, Validator};

use crate::controllers::{request, show};
use crate::ports::CheckSource;
use crate::services::{HostChecks, PermissionBoard, validate_permissions};
use crate::types::{Limits, Owner, PermissionsState};
use crate::usecases::{ReadPermissionSettings, RequestPermissions, ShowPermissions};

pub struct PermissionsFeature {
    state: PermissionsState,
}

impl PermissionsFeature {
    pub const NAME: &'static str = "permissions";
    pub const PATH: &'static str = "/api/permissions";
    pub const REQUEST: &'static str = "/api/permissions/request";

    pub fn new(configuration: Arc<ConfigStore>) -> PermissionsFeature {
        let host = HostChecks::of_this_host();
        let limits = host.limits;
        PermissionsFeature::with(
            configuration,
            Arc::new(host),
            Owner::of_this_process(),
            limits,
        )
    }

    pub fn with(
        configuration: Arc<ConfigStore>,
        source: Arc<dyn CheckSource>,
        owner: Owner,
        limits: Limits,
    ) -> PermissionsFeature {
        let settings = ReadPermissionSettings::new(configuration);
        let board = PermissionBoard::default();
        PermissionsFeature {
            state: PermissionsState {
                show: ShowPermissions::new(
                    settings.clone(),
                    board.clone(),
                    source.clone(),
                    owner.clone(),
                ),
                request: RequestPermissions::new(settings, board, source, owner, limits),
            },
        }
    }

    pub fn show_permissions(&self) -> ShowPermissions {
        self.state.show.clone()
    }

    pub fn request_permissions(&self) -> RequestPermissions {
        self.state.request.clone()
    }
}

impl Feature for PermissionsFeature {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn router(&self) -> Router {
        Router::new()
            .route(Self::PATH, get(show))
            .route(Self::REQUEST, post(request))
            .with_state(self.state.clone())
    }

    fn rules(&self) -> Vec<Rule> {
        vec![
            Rule::needs(
                Method::GET,
                Self::PATH,
                &[Right {
                    area: Area::HostPermissions,
                    action: Action::Read,
                }],
            ),
            Rule::needs(
                Method::POST,
                Self::REQUEST,
                &[Right {
                    area: Area::HostPermissions,
                    action: Action::Update,
                }],
            ),
        ]
    }

    fn validator(&self) -> Option<Validator> {
        Some(validate_permissions)
    }

    fn loops(&self) -> Vec<Loop> {
        let request = self.state.request.clone();
        vec![Box::pin(async move { request.at_start().await })]
    }
}
