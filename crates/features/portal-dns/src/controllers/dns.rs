use axum::Json;
use axum::extract::State;
use axum::http::header::ETAG;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use portal_config::Revision;
use portal_feature::ApiError;

use crate::repositories::{SECTION, write_dns};
use crate::requests::DnsRequest;
use crate::responses::DnsResponse;
use crate::types::DnsContext;

pub async fn show(State(context): State<DnsContext>) -> Response {
    answer(&context)
}

pub async fn change(
    State(context): State<DnsContext>,
    headers: HeaderMap,
    Json(request): Json<DnsRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let target = context
        .configuration
        .read()
        .origins
        .table(SECTION)
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(|| context.configuration.writes_to());
    context
        .configuration
        .update(&target, &revision, |document| {
            write_dns(document, &request);
            Ok(())
        })
        .await?;
    context.runtime.refresh_now();
    Ok(answer(&context))
}

fn answer(context: &DnsContext) -> Response {
    let library = &context.runtime.library;
    let body = DnsResponse::of(&library.settings(), &library.book(), &library.state());
    let mut response = (StatusCode::OK, Json(body)).into_response();
    response
        .headers_mut()
        .insert(ETAG, context.configuration.read().revision.etag());
    response
}
