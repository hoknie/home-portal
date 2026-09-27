use axum::body::Body;
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, Request};
use axum::middleware::Next;
use axum::response::Response;
use portal_feature::ClientAddress;
use portal_model::{DetectedEnvironment, Environment, Environments};
use portal_network::{CurrentNetwork, client_address};
use std::net::SocketAddr;

use super::cookie_value::cookie_value;

const ENVIRONMENT_COOKIE: &str = "portal_environment";

pub async fn decide_environment(
    State(network): State<CurrentNetwork>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let reading = network.run();
    let settings = reading.settings.unwrap_or_default();
    let peer = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(address)| *address);
    let address = client_address(peer, request.headers(), &settings.trusted_proxies);
    let environments = reading.environments.unwrap_or_default();
    let detected = environments.of(address);
    let effective = chosen(&environments, &detected, request.headers());
    request.extensions_mut().insert(effective);
    request.extensions_mut().insert(ClientAddress(address));
    request
        .extensions_mut()
        .insert(DetectedEnvironment::new(detected, &environments));
    next.run(request).await
}

fn chosen(environments: &Environments, detected: &Environment, headers: &HeaderMap) -> Environment {
    let choice = cookie_value(headers, ENVIRONMENT_COOKIE);
    environments.effective(detected, choice)
}
