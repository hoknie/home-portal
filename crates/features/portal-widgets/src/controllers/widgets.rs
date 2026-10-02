use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use portal_feature::{ApiError, Principal};
use portal_model::Environment;

use crate::requests::WidgetPreviewRequest;
use crate::responses::{WidgetActedResponse, WidgetPreviewResponse};
use crate::types::WidgetsState;

pub async fn press_widget_action(
    State(state): State<WidgetsState>,
    Extension(environment): Extension<Environment>,
    principal: Option<Extension<Principal>>,
    Path((widget, action)): Path<(String, String)>,
) -> Result<(StatusCode, Json<WidgetActedResponse>), ApiError> {
    let Some(Extension(principal)) = principal else {
        return Err(ApiError::Unauthorized);
    };
    let run_id = state
        .press
        .run((&widget, &action), &environment, &principal)?;
    Ok((
        StatusCode::ACCEPTED,
        Json(WidgetActedResponse {
            run_id: run_id.map(|id| id.to_string()),
        }),
    ))
}

pub async fn preview_widget(
    State(state): State<WidgetsState>,
    principal: Option<Extension<Principal>>,
    body: Bytes,
) -> Result<Json<WidgetPreviewResponse>, ApiError> {
    let Some(Extension(principal)) = principal else {
        return Err(ApiError::Unauthorized);
    };
    let request: WidgetPreviewRequest = serde_json::from_slice(&body)
        .map_err(|error| ApiError::invalid("body", &error.to_string()))?;
    state
        .preview
        .run(request.asked(), &principal)
        .await
        .map(Json)
}
