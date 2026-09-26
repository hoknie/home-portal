use axum::Extension;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::header::ETAG;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use portal_config::{Revision, Snapshot};
use portal_feature::{ApiError, EventName, PortalEvent, Principal};
use portal_model::Environment;
use time::OffsetDateTime;

use crate::repositories::{
    append, origin, position, published_elsewhere, remove as remove_entry, replace,
};
use crate::requests::ServiceRequest;
use crate::responses::{ServiceResponse, ServicesResponse};
use crate::services::{check_entry, known_of};
use crate::types::{ServiceEntry, ServicesSection, ServicesState, Viewpoint};

pub const UNKNOWN_SERVICE: &str = "no such service";
pub const TAKEN_ID: &str = "is used by another service";
pub const TAKEN_HOST: &str = "is published by another service";

pub async fn list(
    State(state): State<ServicesState>,
    Extension(environment): Extension<Environment>,
) -> Result<Response, ApiError> {
    let snapshot = state.configuration.read();
    let body = services_of(&state, &snapshot, &environment)?;
    Ok(with_revision(StatusCode::OK, &snapshot, Json(body)))
}

pub async fn create(
    State(state): State<ServicesState>,
    Extension(environment): Extension<Environment>,
    principal: Option<Extension<Principal>>,
    headers: HeaderMap,
    Json(request): Json<ServiceRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let entry = checked(request, &state)?;
    let target = state.configuration.writes_to();
    let (_, snapshot) = state
        .configuration
        .update(&target, &revision, |document| {
            if position(document, &entry.id).is_some() {
                return Err(ApiError::invalid("id", TAKEN_ID));
            }
            if published_elsewhere(document, &entry, &entry.id) {
                return Err(ApiError::invalid("proxy.host", TAKEN_HOST));
            }
            append(document, &entry);
            Ok(())
        })
        .await?;
    state.supervisor.reconcile();
    announce(
        &state,
        EventName::ServiceCreated,
        &[
            ("service.id", &entry.id),
            ("service.name", &entry.name),
            ("user.name", &user_of(principal)),
        ],
    );
    let body = ServiceResponse::of(
        entry.clone(),
        viewpoint(&state, &environment),
        state.board.status(&entry.id),
    );
    Ok(with_revision(StatusCode::CREATED, &snapshot, Json(body)))
}

pub async fn update(
    State(state): State<ServicesState>,
    Extension(environment): Extension<Environment>,
    principal: Option<Extension<Principal>>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<ServiceRequest>,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let entry = checked(request, &state)?;
    let target =
        origin(&state.configuration.read(), &id).ok_or(ApiError::NotFound(UNKNOWN_SERVICE))?;
    let (_, snapshot) = state
        .configuration
        .update(&target, &revision, |document| {
            let index = position(document, &id).ok_or(ApiError::NotFound(UNKNOWN_SERVICE))?;
            if entry.id != id && position(document, &entry.id).is_some() {
                return Err(ApiError::invalid("id", TAKEN_ID));
            }
            if published_elsewhere(document, &entry, &id) {
                return Err(ApiError::invalid("proxy.host", TAKEN_HOST));
            }
            replace(document, index, &entry);
            Ok(())
        })
        .await?;
    if entry.id != id {
        state.board.rename(&id, &entry.id);
    }
    state.supervisor.reconcile();
    announce(
        &state,
        EventName::ServiceUpdated,
        &[
            ("service.id", &entry.id),
            ("service.name", &entry.name),
            ("service.previous_id", &id),
            ("user.name", &user_of(principal)),
        ],
    );
    let body = ServiceResponse::of(
        entry.clone(),
        viewpoint(&state, &environment),
        state.board.status(&entry.id),
    );
    Ok(with_revision(StatusCode::OK, &snapshot, Json(body)))
}

pub async fn remove(
    State(state): State<ServicesState>,
    Extension(environment): Extension<Environment>,
    principal: Option<Extension<Principal>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let revision = Revision::from_headers(&headers)?;
    let name = ServicesSection::read(&state.configuration.read().document)
        .ok()
        .and_then(|section| section.services.into_iter().find(|entry| entry.id == id))
        .map(|entry| entry.name)
        .unwrap_or_default();
    let target =
        origin(&state.configuration.read(), &id).ok_or(ApiError::NotFound(UNKNOWN_SERVICE))?;
    let (_, snapshot) = state
        .configuration
        .update(&target, &revision, |document| {
            let index = position(document, &id).ok_or(ApiError::NotFound(UNKNOWN_SERVICE))?;
            remove_entry(document, index);
            Ok(())
        })
        .await?;
    state.supervisor.reconcile();
    announce(
        &state,
        EventName::ServiceDeleted,
        &[
            ("service.id", &id),
            ("service.name", &name),
            ("user.name", &user_of(principal)),
        ],
    );
    let body = services_of(&state, &snapshot, &environment)?;
    Ok(with_revision(StatusCode::OK, &snapshot, Json(body)))
}

fn announce(state: &ServicesState, name: EventName, values: &[(&str, &str)]) {
    state
        .events
        .emit(PortalEvent::of(name, OffsetDateTime::now_utc(), values));
}

fn user_of(principal: Option<Extension<Principal>>) -> String {
    principal
        .map(|Extension(principal)| principal.name)
        .unwrap_or_default()
}

fn checked(request: ServiceRequest, state: &ServicesState) -> Result<ServiceEntry, ApiError> {
    let entry = request.into_entry();
    let known = known_of(&state.configuration.read().document);
    let mut errors = check_entry(&entry, &known);
    if let Some(publication) = &entry.proxy {
        errors.extend(state.publishing.problems(publication));
    }
    if errors.is_empty() {
        Ok(entry)
    } else {
        Err(ApiError::Invalid(errors))
    }
}

fn services_of(
    state: &ServicesState,
    snapshot: &Snapshot,
    environment: &Environment,
) -> Result<ServicesResponse, ApiError> {
    let section = ServicesSection::read(&snapshot.document)
        .map_err(|message| ApiError::Internal(format!("services section: {message}")))?;
    let services = section
        .services
        .into_iter()
        .filter(|entry| entry.visible_to(environment))
        .map(|entry| {
            let status = state.board.status(&entry.id);
            ServiceResponse::of(entry, viewpoint(state, environment), status)
        })
        .collect();
    Ok(ServicesResponse { services })
}

fn viewpoint<'a>(state: &'a ServicesState, environment: &'a Environment) -> Viewpoint<'a> {
    Viewpoint {
        environment,
        host: state.supervisor.host(),
        publishing: state.publishing.https_port(),
    }
}

fn with_revision(status: StatusCode, snapshot: &Snapshot, body: impl IntoResponse) -> Response {
    let mut response = (status, body).into_response();
    response
        .headers_mut()
        .insert(ETAG, snapshot.revision.etag());
    response
}
