use axum::extract::{Path, State};
use axum::http::header::ETAG;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use portal_config::{Revision, Revisioned};
use portal_feature::{ApiError, Principal};

use crate::requests::GroupWriteRequest;
use crate::responses::GroupsResponse;
use crate::types::{AuthState, GroupsView};

pub async fn list_groups(State(state): State<AuthState>) -> Result<Response, ApiError> {
    Ok(answer(StatusCode::OK, state.list_groups.run()?))
}

pub async fn create_group(
    State(state): State<AuthState>,
    Extension(principal): Extension<Principal>,
    headers: HeaderMap,
    Json(request): Json<GroupWriteRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let created = state
        .create_group
        .run(&principal, &request.name, &request.rights, &revision)
        .await?;
    Ok(answer(StatusCode::CREATED, created))
}

pub async fn change_group_entry(
    State(state): State<AuthState>,
    Extension(principal): Extension<Principal>,
    Path(name): Path<String>,
    headers: HeaderMap,
    Json(request): Json<GroupWriteRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let changed = state
        .change_group_entry
        .run(&principal, &name, &request.name, &request.rights, &revision)
        .await?;
    Ok(answer(StatusCode::OK, changed))
}

pub async fn delete_group(
    State(state): State<AuthState>,
    Extension(principal): Extension<Principal>,
    Path(name): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let deleted = state.delete_group.run(&principal, &name, &revision).await?;
    Ok(answer(StatusCode::OK, deleted))
}

fn answer(status: StatusCode, view: Revisioned<GroupsView>) -> Response {
    let mut response = (status, Json(GroupsResponse::of(&view.value))).into_response();
    response.headers_mut().insert(ETAG, view.revision.etag());
    response
}
