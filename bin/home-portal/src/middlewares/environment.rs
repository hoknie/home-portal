use std::net::SocketAddr;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, Request};
use axum::middleware::Next;
use axum::response::Response;
use portal_config::ConfigStore;
use portal_model::{DetectedEnvironment, Environment, Environments};
use portal_network::{client_address, read_environments, read_network};

use super::cookie_value::cookie_value;

const ENVIRONMENT_COOKIE: &str = "portal_environment";

pub async fn decide_environment(
    State(configuration): State<Arc<ConfigStore>>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let snapshot = configuration.read();
    let settings = read_network(&snapshot.document).unwrap_or_default();
    let peer = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(address)| *address);
    let address = client_address(peer, request.headers(), &settings.trusted_proxies);
    let environments = read_environments(&snapshot.document).unwrap_or_default();
    let detected = environments.of(address);
    let effective = chosen(&environments, &detected, request.headers());
    request.extensions_mut().insert(effective);
    request
        .extensions_mut()
        .insert(DetectedEnvironment::new(detected, &environments));
    next.run(request).await
}

fn chosen(environments: &Environments, detected: &Environment, headers: &HeaderMap) -> Environment {
    let choice = cookie_value(headers, ENVIRONMENT_COOKIE);
    environments.effective(detected, choice)
}
