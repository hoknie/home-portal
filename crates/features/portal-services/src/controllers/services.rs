use axum::Extension;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::header::ETAG;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use portal_config::{Revision, Revisioned};
use portal_feature::{ApiError, Principal};
use portal_model::Environment;

use crate::requests::ServiceRequest;
use crate::responses::{ServiceResponse, ServicesResponse};
use crate::types::{ServicesState, ShownService, Viewpoint};

pub async fn list(
    State(state): State<ServicesState>,
    Extension(environment): Extension<Environment>,
) -> Result<Response, ApiError> {
    let listed = state.list.run(&environment)?;
    Ok(listing(StatusCode::OK, listed, &environment))
}

pub async fn create(
    State(state): State<ServicesState>,
    Extension(environment): Extension<Environment>,
    principal: Option<Extension<Principal>>,
    headers: HeaderMap,
    Json(request): Json<ServiceRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let created = state
        .create
        .run(request.into_entry(), &revision, &user_of(principal))
        .await?;
    Ok(one(StatusCode::CREATED, created, &environment))
}

pub async fn update(
    State(state): State<ServicesState>,
    Extension(environment): Extension<Environment>,
    principal: Option<Extension<Principal>>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<ServiceRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let changed = state
        .change
        .run(&id, request.into_entry(), &revision, &user_of(principal))
        .await?;
    Ok(one(StatusCode::OK, changed, &environment))
}

pub async fn remove(
    State(state): State<ServicesState>,
    Extension(environment): Extension<Environment>,
    principal: Option<Extension<Principal>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let remaining = state
        .delete
        .run(&id, &revision, &environment, &user_of(principal))
        .await?;
    Ok(listing(StatusCode::OK, remaining, &environment))
}

fn user_of(principal: Option<Extension<Principal>>) -> String {
    principal
        .map(|Extension(principal)| principal.name)
        .unwrap_or_default()
}

fn response_of(shown: ShownService, environment: &Environment) -> ServiceResponse {
    let ShownService {
        entry,
        status,
        host,
        publishing,
    } = shown;
    let viewpoint = Viewpoint {
        environment,
        host: &host,
        publishing,
    };
    ServiceResponse::of(entry, viewpoint, status)
}

fn one(status: StatusCode, shown: Revisioned<ShownService>, environment: &Environment) -> Response {
    let body = response_of(shown.value, environment);
    with_revision(status, &shown.revision, Json(body))
}

fn listing(
    status: StatusCode,
    listed: Revisioned<Vec<ShownService>>,
    environment: &Environment,
) -> Response {
    let services = listed
        .value
        .into_iter()
        .map(|shown| response_of(shown, environment))
        .collect();
    with_revision(
        status,
        &listed.revision,
        Json(ServicesResponse { services }),
    )
}

fn with_revision(status: StatusCode, revision: &Revision, body: impl IntoResponse) -> Response {
    let mut response = (status, body).into_response();
    response.headers_mut().insert(ETAG, revision.etag());
    response
}
