use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use portal_feature::{ApiError, FieldError};
use url::Url;

use crate::helpers::{SVG, extension_of, inert_headers, svg_is_inert};
use crate::requests::PreviewRequest;
use crate::services::Icons;
use crate::types::IconSource;

pub const FIELD: &str = "icon";
pub const LUCIDE_PREVIEW: &str = "a lucide icon is drawn by the interface";
pub const AUTO_WITHOUT_ADDRESS: &str = "auto needs the service's http or https address";
pub const NO_STORE: &str = "no-store";

pub async fn preview(
    State(icons): State<Arc<Icons>>,
    Json(request): Json<PreviewRequest>,
) -> Result<Response, ApiError> {
    let source = IconSource::parse(request.icon.trim()).map_err(invalid)?;
    let address = request
        .url
        .as_deref()
        .and_then(|url| Url::parse(url.trim()).ok())
        .filter(|url| matches!(url.scheme(), "http" | "https") && url.host().is_some());
    match source {
        IconSource::Lucide(_) => return Err(invalid(LUCIDE_PREVIEW)),
        IconSource::Discovered if address.is_none() => return Err(invalid(AUTO_WITHOUT_ADDRESS)),
        _ => {}
    }
    let icon = icons
        .preview(&source, address.as_ref())
        .await
        .map_err(invalid)?;
    if icon.content_type == SVG && !svg_is_inert(&icon.bytes) {
        return Err(invalid(ACTIVE_IMAGE));
    }
    let extension = extension_of(&icon.content_type);
    let mut response = (StatusCode::OK, icon.bytes).into_response();
    let headers = response.headers_mut();
    inert_headers(headers, extension);
    if let Ok(content_type) = HeaderValue::from_str(&icon.content_type) {
        headers.insert(CONTENT_TYPE, content_type);
    }
    headers.insert(CACHE_CONTROL, HeaderValue::from_static(NO_STORE));
    Ok(response)
}

pub const ACTIVE_IMAGE: &str = "the image can run script, so it is not used";

fn invalid(problem: impl ToString) -> ApiError {
    ApiError::Invalid(vec![FieldError::new(FIELD, problem.to_string())])
}
