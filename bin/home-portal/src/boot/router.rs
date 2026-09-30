use std::sync::Arc;

use axum::Router;
use axum::http::Method;
use axum::middleware;
use axum::routing::{any, post};
use portal_network::CurrentNetwork;
use portal_web::CurrentInterface;

use crate::controllers::{RESTART_PATH, not_found, restart};
use portal_feature::{Action, Area, Right, Rule};

use crate::middlewares::{
    deadline, decide_environment, decide_language, json_only, require_right, require_session,
    same_origin, security_headers,
};
use crate::types::{Registry, RuleBook};

pub const API_ROOT: &str = "/api";
pub const API_ANY_PATH: &str = "/api/{*rest}";
pub const RESTART_RIGHTS: &[Right] = &[Right::new(Area::Portal, Action::Update)];

pub fn rule_book(registry: &Registry) -> RuleBook {
    RuleBook::of(
        registry
            .features
            .iter()
            .flat_map(|feature| feature.rules())
            .chain([
                Rule::signed(Method::GET, portal_widget::DATA_PATH),
                Rule::needs(Method::POST, RESTART_PATH, RESTART_RIGHTS),
            ]),
    )
}

pub fn assemble(registry: &Registry) -> Router {
    let configuration = registry.configuration.clone();
    let protected = registry
        .features
        .iter()
        .fold(Router::new(), |router, feature| {
            router.merge(feature.router())
        })
        .merge(portal_widget::widgets_router(registry.widgets.clone()))
        .merge(
            Router::new()
                .route(RESTART_PATH, post(restart))
                .with_state(registry.restart.clone()),
        )
        .route_layer(middleware::from_fn_with_state(
            Arc::new(rule_book(registry)),
            require_right,
        ))
        .route_layer(middleware::from_fn_with_state(
            registry.gate.clone(),
            require_session,
        ));
    let public = registry
        .features
        .iter()
        .fold(Router::new(), |router, feature| {
            router.merge(feature.public_router())
        });
    protected
        .merge(public)
        .route(API_ROOT, any(not_found))
        .route(API_ANY_PATH, any(not_found))
        .fallback_service(portal_web::interface(registry.interface.clone()))
        .layer(middleware::from_fn(deadline))
        .layer(middleware::from_fn(json_only))
        .layer(middleware::from_fn(same_origin))
        .layer(middleware::from_fn_with_state(
            CurrentNetwork::new(configuration.clone()),
            decide_environment,
        ))
        .layer(middleware::from_fn_with_state(
            CurrentInterface::new(configuration),
            decide_language,
        ))
        .layer(middleware::from_fn(security_headers))
}
