use axum::Json;
use axum::extract::{Path, State};
use axum::http::header::ETAG;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use portal_config::{Revision, Revisioned};
use portal_feature::ApiError;
use serde_json::Value;

use crate::requests::{RulesRequest, TestRequest};
use crate::responses::{DeliveryResponse, NotificationsResponse};
use crate::types::{NotificationsState, NotificationsView};

pub async fn list(State(state): State<NotificationsState>) -> Response {
    answer(state.list.run())
}

pub async fn change_rules(
    State(state): State<NotificationsState>,
    headers: HeaderMap,
    Json(request): Json<RulesRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    Ok(answer(
        state
            .change_rules
            .run(request.into_rules(), &revision)
            .await?,
    ))
}

pub async fn change_channel(
    State(state): State<NotificationsState>,
    Path(name): Path<String>,
    headers: HeaderMap,
    Json(settings): Json<Value>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    Ok(answer(
        state
            .change_channel
            .run(&name, &settings, &revision)
            .await?,
    ))
}

pub async fn send_test(
    State(state): State<NotificationsState>,
    Json(request): Json<TestRequest>,
) -> Result<Json<DeliveryResponse>, ApiError> {
    let delivery = state.send_test.run(&request.channel).await?;
    Ok(Json(DeliveryResponse::of(&delivery)))
}

fn answer(view: Revisioned<NotificationsView>) -> Response {
    let mut response =
        (StatusCode::OK, Json(NotificationsResponse::of(&view.value))).into_response();
    response.headers_mut().insert(ETAG, view.revision.etag());
    response
}
