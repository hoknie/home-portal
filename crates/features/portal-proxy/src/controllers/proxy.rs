use axum::Json;
use axum::extract::State;
use axum::http::header::ETAG;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use portal_config::{Revision, Revisioned};
use portal_feature::ApiError;

use crate::requests::ProxySettingsRequest;
use crate::responses::ProxyResponse;
use crate::types::{ProxyState, ProxyView};

pub async fn show(State(state): State<ProxyState>) -> Response {
    answer(state.show.run(), StatusCode::OK)
}

pub async fn apply(State(state): State<ProxyState>) -> Result<Response, ApiError> {
    Ok(answer(state.apply.run().await?, StatusCode::OK))
}

pub async fn change(
    State(state): State<ProxyState>,
    headers: HeaderMap,
    Json(request): Json<ProxySettingsRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let changed = state.change.run(&request.into_choice(), &revision).await?;
    Ok(answer(changed, StatusCode::OK))
}

pub fn answer(view: Revisioned<ProxyView>, status: StatusCode) -> Response {
    let mut response = (status, Json(ProxyResponse::of(view.value))).into_response();
    response.headers_mut().insert(ETAG, view.revision.etag());
    response
}
