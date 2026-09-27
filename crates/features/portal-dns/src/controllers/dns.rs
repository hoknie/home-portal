use axum::Json;
use axum::extract::State;
use axum::http::header::ETAG;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use portal_config::{Revision, Revisioned};
use portal_feature::ApiError;

use crate::requests::DnsRequest;
use crate::responses::DnsResponse;
use crate::types::{DnsContext, DnsView};

pub async fn show(State(context): State<DnsContext>) -> Response {
    answer(context.show.run())
}

pub async fn change(
    State(context): State<DnsContext>,
    headers: HeaderMap,
    Json(request): Json<DnsRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let changed = context
        .change
        .run(&request.into_choice(), &revision)
        .await?;
    Ok(answer(changed))
}

fn answer(view: Revisioned<DnsView>) -> Response {
    let shown = &view.value;
    let body = DnsResponse::of(&shown.settings, &shown.book, &shown.state);
    let mut response = (StatusCode::OK, Json(body)).into_response();
    response.headers_mut().insert(ETAG, view.revision.etag());
    response
}
