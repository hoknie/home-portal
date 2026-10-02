use std::sync::Arc;

use axum::Router;
use axum::middleware;
use axum::routing::{any, get};
use portal_model::Language;
use portal_web::AssetSource;

use super::board::FailureBoard;
use portal_health::HealthFeature;

use crate::boot::{API_ANY_PATH, API_ROOT};
use crate::controllers::{FAILURE_PATH, failed_health, failure_report, not_started, page_or_fatal};
use crate::middlewares::{deadline, decide_language_from, same_origin, security_headers};

pub fn failure_router(
    board: FailureBoard,
    interface: Arc<dyn AssetSource>,
    language: Language,
) -> Router {
    let report = Router::new()
        .route(FAILURE_PATH, get(failure_report))
        .with_state(board);
    let pages = Router::new().fallback(page_or_fatal).with_state(interface);
    Router::new()
        .route(HealthFeature::PATH, get(failed_health))
        .merge(report)
        .route(API_ROOT, any(not_started))
        .route(API_ANY_PATH, any(not_started))
        .merge(pages)
        .layer(middleware::from_fn(deadline))
        .layer(middleware::from_fn(same_origin))
        .layer(middleware::from_fn_with_state(
            language,
            decide_language_from,
        ))
        .layer(middleware::from_fn(security_headers))
}
