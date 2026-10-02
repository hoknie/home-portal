use axum::extract::{Path, State};
use axum::http::header::ETAG;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use portal_config::Revision;
use portal_feature::{ApiError, Principal};

use crate::requests::LibraryRequest;
use crate::responses::{LibraryResponse, LibraryWidgetView};
use crate::types::DashboardState;

pub async fn list_library(State(state): State<DashboardState>) -> Result<Response, ApiError> {
    let listed = state.library.list()?;
    let mut response = Json(LibraryResponse {
        widgets: listed
            .value
            .into_iter()
            .map(|(widget, placed)| LibraryWidgetView::of(widget, placed))
            .collect(),
    })
    .into_response();
    response.headers_mut().insert(ETAG, listed.revision.etag());
    Ok(response)
}

pub async fn create_library(
    State(state): State<DashboardState>,
    principal: Option<Extension<Principal>>,
    headers: HeaderMap,
    Json(request): Json<LibraryRequest>,
) -> Result<Response, ApiError> {
    save(state, None, principal, headers, request)
        .await
        .map(|response| (StatusCode::CREATED, response).into_response())
}

pub async fn update_library(
    State(state): State<DashboardState>,
    principal: Option<Extension<Principal>>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<LibraryRequest>,
) -> Result<Response, ApiError> {
    save(state, Some(id), principal, headers, request).await
}

pub async fn delete_library(
    State(state): State<DashboardState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let next = state.library.delete(&id, &revision).await?;
    let mut response = StatusCode::NO_CONTENT.into_response();
    response.headers_mut().insert(ETAG, next.etag());
    Ok(response)
}

async fn save(
    state: DashboardState,
    existing: Option<String>,
    principal: Option<Extension<Principal>>,
    headers: HeaderMap,
    request: LibraryRequest,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let rights = principal
        .map(|Extension(principal)| principal.rights)
        .unwrap_or_default();
    let saved = state
        .library
        .save(
            existing.as_deref(),
            request.into_instance(),
            (&revision, &rights),
        )
        .await?;
    let (widget, placed) = saved.value;
    let mut response = Json(LibraryWidgetView::of(widget, placed)).into_response();
    response.headers_mut().insert(ETAG, saved.revision.etag());
    Ok(response)
}
