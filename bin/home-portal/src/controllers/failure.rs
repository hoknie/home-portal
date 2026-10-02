use std::net::SocketAddr;
use std::sync::Arc;

use axum::Extension;
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode, Uri};
use axum::response::{IntoResponse, Json, Redirect, Response};
use portal_model::Language;
use portal_web::{AssetSource, answer};

use crate::failures::{FailureBoard, report};
use crate::responses::FailureReportResponse;

pub const FAILURE_PATH: &str = "/api/portal/failure";
pub const FATAL_PAGE: &str = "/fatal/";
pub const FATAL_PATH: &str = "/fatal";
pub const HOME_PAGE: &str = "/";
pub const FAILED: &str = "failed";
pub const STATE_HEADER: HeaderName = HeaderName::from_static("portal-state");
pub const NOT_STARTED: &str = "the portal did not start; open /fatal/ to see why";

pub async fn failed_health() -> (StatusCode, &'static str) {
    (StatusCode::SERVICE_UNAVAILABLE, FAILED)
}

pub async fn failure_report(
    State(board): State<FailureBoard>,
    peer: Option<Extension<ConnectInfo<SocketAddr>>>,
    headers: HeaderMap,
) -> Json<FailureReportResponse> {
    let peer = peer.map(|Extension(ConnectInfo(address))| address);
    Json(report(&board, peer, &headers))
}

pub async fn not_started() -> Response {
    let mut response = (StatusCode::SERVICE_UNAVAILABLE, NOT_STARTED).into_response();
    response
        .headers_mut()
        .insert(STATE_HEADER, HeaderValue::from_static(FAILED));
    response
}

pub async fn page_or_fatal(
    State(source): State<Arc<dyn AssetSource>>,
    language: Option<Extension<Language>>,
    uri: Uri,
) -> Response {
    let path = uri.path();
    let file = path
        .rsplit('/')
        .next()
        .is_some_and(|segment| segment.contains('.'));
    if path.starts_with(FATAL_PAGE) || file {
        let language = language
            .map(|Extension(language)| language)
            .unwrap_or_default();
        return answer(source.as_ref(), language, path);
    }
    Redirect::temporary(FATAL_PAGE).into_response()
}

pub async fn fatal_to_home() -> Redirect {
    Redirect::temporary(HOME_PAGE)
}
