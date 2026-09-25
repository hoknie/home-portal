use std::sync::Arc;

use axum::Extension;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE, ETAG, IF_NONE_MATCH};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use portal_feature::ApiError;
use portal_model::Environment;

use crate::services::Icons;
use crate::types::{IconState, StoredIcon};

pub const MISSING_ICON: &str = "this service has no icon of its own";
pub const CACHE: &str = "private, max-age=3600";

pub async fn serve_icon(
    State(icons): State<Arc<Icons>>,
    Path(service): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    answer(icons.icon_of(&service), &headers)
}

pub async fn serve_public_icon(
    State(icons): State<Arc<Icons>>,
    Extension(environment): Extension<Environment>,
    Path(service): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    answer(
        icons.public_icon_of(&service, environment.as_str()),
        &headers,
    )
}

fn answer(icon: Option<StoredIcon>, headers: &HeaderMap) -> Result<Response, ApiError> {
    let icon = icon.ok_or(ApiError::NotFound(MISSING_ICON))?;
    let tag = format!("\"{}\"", icon.digest);
    if headers
        .get(IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.split(',').any(|candidate| candidate.trim() == tag))
    {
        return Ok(StatusCode::NOT_MODIFIED.into_response());
    }
    let mut response = (StatusCode::OK, icon.bytes).into_response();
    let headers = response.headers_mut();
    if let Ok(content_type) = HeaderValue::from_str(&icon.content_type) {
        headers.insert(CONTENT_TYPE, content_type);
    }
    if let Ok(tag) = HeaderValue::from_str(&tag) {
        headers.insert(ETAG, tag);
    }
    headers.insert(CACHE_CONTROL, HeaderValue::from_static(CACHE));
    Ok(response)
}

pub async fn state(State(icons): State<Arc<Icons>>) -> Json<Vec<IconState>> {
    Json(icons.described())
}
