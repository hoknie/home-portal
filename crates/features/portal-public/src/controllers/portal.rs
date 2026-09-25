use axum::extract::{Path, State};
use axum::{Extension, Json};
use portal_feature::ApiError;
use portal_model::{DetectedEnvironment, Environment};
use portal_widget::WidgetData;

use crate::responses::PortalResponse;
use crate::types::PublicState;

pub async fn portal(
    State(state): State<PublicState>,
    Extension(environment): Extension<Environment>,
    Extension(detected): Extension<DetectedEnvironment>,
) -> Json<PortalResponse> {
    Json(PortalResponse {
        switchable: detected.switchable(),
        environments: detected.switchable().then_some(detected.choices),
        detected: detected.environment,
        sections: state.layout.public_sections(&environment).await,
        services: state.services.public_services(&environment),
        widgets: state.layout.public_widgets(&environment).await,
        environment,
    })
}

pub async fn widget_data(
    State(state): State<PublicState>,
    Extension(environment): Extension<Environment>,
    Path(id): Path<String>,
) -> Result<Json<WidgetData>, ApiError> {
    state.layout.public_data(&id, &environment).await.map(Json)
}
