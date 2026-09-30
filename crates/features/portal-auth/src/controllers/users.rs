use axum::extract::{Path, State};
use axum::http::header::ETAG;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use portal_config::{Revision, Revisioned};
use portal_feature::{ApiError, Principal};

use crate::helpers::session_token;
use crate::requests::{GroupRequest, NewUserRequest, PasswordRequest};
use crate::responses::UsersResponse;
use crate::types::{AuthState, Caller, UsersView};

pub async fn list_users(
    State(state): State<AuthState>,
    Extension(principal): Extension<Principal>,
) -> Result<Response, ApiError> {
    Ok(answer(
        StatusCode::OK,
        state.list_users.run(&principal.name)?,
    ))
}

pub async fn create_user(
    State(state): State<AuthState>,
    Extension(principal): Extension<Principal>,
    headers: HeaderMap,
    Json(request): Json<NewUserRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let created = state
        .create_user
        .run(
            &principal,
            &request.name,
            request.password,
            request.group,
            &revision,
        )
        .await?;
    Ok(answer(StatusCode::CREATED, created))
}

pub async fn change_password(
    State(state): State<AuthState>,
    Extension(principal): Extension<Principal>,
    Path(name): Path<String>,
    headers: HeaderMap,
    Json(request): Json<PasswordRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let caller = Caller {
        principal,
        token: session_token(&headers),
    };
    let changed = state
        .change_password
        .run(&caller, &name, request.password, &revision)
        .await?;
    Ok(answer(StatusCode::OK, changed))
}

pub async fn delete_user(
    State(state): State<AuthState>,
    Extension(principal): Extension<Principal>,
    Path(name): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let deleted = state.delete_user.run(&principal, &name, &revision).await?;
    Ok(answer(StatusCode::OK, deleted))
}

pub async fn change_group(
    State(state): State<AuthState>,
    Extension(principal): Extension<Principal>,
    Path(name): Path<String>,
    headers: HeaderMap,
    Json(request): Json<GroupRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let changed = state
        .change_group
        .run(&principal, &name, request.group, &revision)
        .await?;
    Ok(answer(StatusCode::OK, changed))
}

fn answer(status: StatusCode, view: Revisioned<UsersView>) -> Response {
    let mut response = (status, Json(UsersResponse::of(view.value))).into_response();
    response.headers_mut().insert(ETAG, view.revision.etag());
    response
}
