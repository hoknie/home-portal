use axum::Json;
use axum::extract::{Path, State};
use axum::http::header::ETAG;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use portal_config::{Revision, Snapshot};
use portal_feature::ApiError;

use crate::repositories::{append, origin, position, remove as remove_entry, replace};
use crate::requests::AutomationRequest;
use crate::responses::{AutomationResponse, AutomationsResponse};
use crate::services::decoded;
use crate::types::{Automation, AutomationsState};

pub const UNKNOWN_AUTOMATION: &str = "no such automation";
pub const TAKEN_ID: &str = "is used by another automation";

pub async fn list(State(state): State<AutomationsState>) -> Response {
    let snapshot = state.configuration.read();
    let body = listed(&state, &snapshot);
    with_revision(StatusCode::OK, &snapshot, Json(body))
}

pub async fn create(
    State(state): State<AutomationsState>,
    headers: HeaderMap,
    Json(request): Json<AutomationRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let automation = checked(request)?;
    let target = state.configuration.writes_to();
    let (_, snapshot) = state
        .configuration
        .update(&target, &revision, |document| {
            if position(document, &automation.id).is_some() {
                return Err(ApiError::invalid("id", TAKEN_ID));
            }
            append(document, &automation);
            Ok(())
        })
        .await?;
    state.sink.cache.refresh(&snapshot.document);
    let body = AutomationResponse::of(&automation, None, None);
    Ok(with_revision(StatusCode::CREATED, &snapshot, Json(body)))
}

pub async fn update(
    State(state): State<AutomationsState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<AutomationRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let automation = checked(request)?;
    let target =
        origin(&state.configuration.read(), &id).ok_or(ApiError::NotFound(UNKNOWN_AUTOMATION))?;
    let (_, snapshot) = state
        .configuration
        .update(&target, &revision, |document| {
            let index = position(document, &id).ok_or(ApiError::NotFound(UNKNOWN_AUTOMATION))?;
            if automation.id != id && position(document, &automation.id).is_some() {
                return Err(ApiError::invalid("id", TAKEN_ID));
            }
            replace(document, index, &automation);
            Ok(())
        })
        .await?;
    state.sink.cache.refresh(&snapshot.document);
    let last = state.sink.journal.last_of(&automation.id);
    let active = state.sink.active.of_automation(&automation.id);
    let body = AutomationResponse::of(&automation, last.as_ref(), active.as_ref());
    Ok(with_revision(StatusCode::OK, &snapshot, Json(body)))
}

pub async fn remove(
    State(state): State<AutomationsState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let target =
        origin(&state.configuration.read(), &id).ok_or(ApiError::NotFound(UNKNOWN_AUTOMATION))?;
    let (_, snapshot) = state
        .configuration
        .update(&target, &revision, |document| {
            let index = position(document, &id).ok_or(ApiError::NotFound(UNKNOWN_AUTOMATION))?;
            remove_entry(document, index);
            Ok(())
        })
        .await?;
    state.sink.cache.refresh(&snapshot.document);
    let body = listed(&state, &snapshot);
    Ok(with_revision(StatusCode::OK, &snapshot, Json(body)))
}

fn checked(request: AutomationRequest) -> Result<Automation, ApiError> {
    Automation::decode(&request.into_raw()).map_err(ApiError::Invalid)
}

fn listed(state: &AutomationsState, snapshot: &Snapshot) -> AutomationsResponse {
    AutomationsResponse {
        automations: decoded(&snapshot.document)
            .iter()
            .map(|automation| {
                let last = state.sink.journal.last_of(&automation.id);
                let active = state.sink.active.of_automation(&automation.id);
                AutomationResponse::of(automation, last.as_ref(), active.as_ref())
            })
            .collect(),
    }
}

fn with_revision(status: StatusCode, snapshot: &Snapshot, body: impl IntoResponse) -> Response {
    let mut response = (status, body).into_response();
    response
        .headers_mut()
        .insert(ETAG, snapshot.revision.etag());
    response
}
