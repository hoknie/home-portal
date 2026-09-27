use axum::Json;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::http::header::ETAG;
use axum::response::{IntoResponse, Response};
use portal_config::{Revision, Revisioned};
use portal_feature::{ApiError, Module};

use crate::requests::SwitchRequest;
use crate::responses::ModulesResponse;
use crate::types::{ModuleView, ModulesState};

pub const UNKNOWN: &str = "no such module";

pub async fn show(State(state): State<ModulesState>) -> Result<Response, ApiError> {
    Ok(answer(state.current.run()?))
}

pub async fn switch(
    State(state): State<ModulesState>,
    Path(name): Path<String>,
    headers: HeaderMap,
    Json(request): Json<SwitchRequest>,
) -> Result<Response, ApiError> {
    let module = Module::from_name(&name).ok_or(ApiError::NotFound(UNKNOWN))?;
    let revision = Revision::from_headers(&headers)?;
    let switched = state.switch.run(module, request.enabled, &revision).await?;
    Ok(answer(switched))
}

fn answer(views: Revisioned<Vec<ModuleView>>) -> Response {
    let mut response = Json(ModulesResponse::of(&views.value)).into_response();
    response.headers_mut().insert(ETAG, views.revision.etag());
    response
}
