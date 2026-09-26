use axum::Router;
use axum::middleware;
use axum::routing::{any, post};

use crate::controllers::{RESTART_PATH, not_found, restart};
use crate::middlewares::{decide_environment, decide_language, json_only, require_session};
use crate::types::Registry;

pub const API_ROOT: &str = "/api";
pub const API_ANY_PATH: &str = "/api/{*rest}";

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
        .layer(middleware::from_fn(json_only))
        .layer(middleware::from_fn_with_state(
            configuration.clone(),
            decide_environment,
        ))
        .layer(middleware::from_fn_with_state(
            configuration,
            decide_language,
        ))
}
