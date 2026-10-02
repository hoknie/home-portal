use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Extension, Json, Router};
use portal_feature::ApiError;
use portal_model::Environment;
use serde_json::json;

use crate::services::WidgetRegistry;
use crate::types::WidgetAnswer;

pub const DATA_PATH: &str = "/api/widgets/{id}/data";

pub fn widgets_router(registry: Arc<WidgetRegistry>) -> Router {
    Router::new()
        .route(DATA_PATH, get(data))
        .with_state(registry)
}

async fn data(
    State(registry): State<Arc<WidgetRegistry>>,
    Extension(environment): Extension<Environment>,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    registry.data(&id, &environment).await.map(answered)
}

pub fn answered(answer: WidgetAnswer) -> Response {
    match answer {
        WidgetAnswer::Ready(data) => Json(data).into_response(),
        WidgetAnswer::Refreshing => {
            (StatusCode::ACCEPTED, Json(json!({ "refreshing": true }))).into_response()
        }
    }
}
