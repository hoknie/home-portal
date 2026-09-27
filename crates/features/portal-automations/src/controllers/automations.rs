use axum::Json;
use axum::extract::{Path, State};
use axum::http::header::ETAG;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use portal_config::{Revision, Revisioned};
use portal_feature::ApiError;

use crate::requests::AutomationRequest;
use crate::responses::{AutomationResponse, AutomationsResponse};
use crate::types::{AutomationView, AutomationsState};

pub async fn list(State(state): State<AutomationsState>) -> Response {
    listing(StatusCode::OK, state.list.run())
}

pub async fn create(
    State(state): State<AutomationsState>,
    headers: HeaderMap,
    Json(request): Json<AutomationRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let created = state.create.run(&request.into_raw(), &revision).await?;
    Ok(one(StatusCode::CREATED, created))
}

pub async fn update(
    State(state): State<AutomationsState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<AutomationRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let changed = state
        .change
        .run(&id, &request.into_raw(), &revision)
        .await?;
    Ok(one(StatusCode::OK, changed))
}

pub async fn remove(
    State(state): State<AutomationsState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    Ok(listing(
        StatusCode::OK,
        state.delete.run(&id, &revision).await?,
    ))
}

fn response_of(view: &AutomationView) -> AutomationResponse {
    AutomationResponse::of(&view.automation, view.last.as_ref(), view.active.as_ref())
}

fn one(status: StatusCode, view: Revisioned<AutomationView>) -> Response {
    with_revision(status, &view.revision, Json(response_of(&view.value)))
}

fn listing(status: StatusCode, views: Revisioned<Vec<AutomationView>>) -> Response {
    let body = AutomationsResponse {
        automations: views.value.iter().map(response_of).collect(),
    };
    with_revision(status, &views.revision, Json(body))
}

fn with_revision(status: StatusCode, revision: &Revision, body: impl IntoResponse) -> Response {
    let mut response = (status, body).into_response();
    response.headers_mut().insert(ETAG, revision.etag());
    response
}
