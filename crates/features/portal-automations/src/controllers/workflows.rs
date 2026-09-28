use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::header::ETAG;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use portal_config::{Revision, Revisioned};
use portal_feature::{ApiError, Principal};

use super::runs::name_of;
use crate::requests::{WorkflowRequest, WorkflowRunRequest};
use crate::responses::{
    QueuedResponse, WorkflowCatalogueResponse, WorkflowResponse, WorkflowsResponse,
};
use crate::types::{AutomationsState, WorkflowView};

pub async fn list_workflows(State(state): State<AutomationsState>) -> Response {
    listing(StatusCode::OK, state.workflows.list.run())
}

pub async fn create_workflow(
    State(state): State<AutomationsState>,
    headers: HeaderMap,
    Json(request): Json<WorkflowRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let raw = request.into_raw().map_err(ApiError::Invalid)?;
    let created = state.workflows.create.run(raw, &revision).await?;
    Ok(one(StatusCode::CREATED, created))
}

pub async fn update_workflow(
    State(state): State<AutomationsState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<WorkflowRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let raw = request.into_raw().map_err(ApiError::Invalid)?;
    let changed = state.workflows.change.run(&id, raw, &revision).await?;
    Ok(one(StatusCode::OK, changed))
}

pub async fn delete_workflow(
    State(state): State<AutomationsState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    Ok(listing(
        StatusCode::OK,
        state.workflows.delete.run(&id, &revision).await?,
    ))
}

pub async fn run_workflow(
    State(state): State<AutomationsState>,
    principal: Option<Extension<Principal>>,
    Path(id): Path<String>,
    body: Bytes,
) -> Result<(StatusCode, Json<QueuedResponse>), ApiError> {
    let inputs = WorkflowRunRequest::parse(&body)
        .map_err(ApiError::BadRequest)?
        .inputs;
    let run_id = state.workflows.run.run(&id, inputs, &name_of(principal))?;
    Ok((
        StatusCode::ACCEPTED,
        Json(QueuedResponse {
            run_id: run_id.to_string(),
        }),
    ))
}

pub async fn workflow_catalogue(
    State(state): State<AutomationsState>,
) -> Json<WorkflowCatalogueResponse> {
    Json(WorkflowCatalogueResponse::of(
        &state.workflows.catalogue.run(),
    ))
}

fn one(status: StatusCode, view: Revisioned<WorkflowView>) -> Response {
    with_revision(
        status,
        &view.revision,
        Json(WorkflowResponse::of(&view.value)),
    )
}

fn listing(status: StatusCode, views: Revisioned<Vec<WorkflowView>>) -> Response {
    let body = WorkflowsResponse {
        workflows: views.value.iter().map(WorkflowResponse::of).collect(),
    };
    with_revision(status, &views.revision, Json(body))
}

fn with_revision(status: StatusCode, revision: &Revision, body: impl IntoResponse) -> Response {
    let mut response = (status, body).into_response();
    response.headers_mut().insert(ETAG, revision.etag());
    response
}
