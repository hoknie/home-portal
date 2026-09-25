use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Extension, Json, Router};
use portal_feature::ApiError;
use portal_model::Environment;

use crate::services::WidgetRegistry;
use crate::types::WidgetData;

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
) -> Result<Json<WidgetData>, ApiError> {
    registry.data(&id, &environment).await.map(Json)
}
