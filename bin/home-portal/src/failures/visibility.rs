use std::net::SocketAddr;

use axum::http::HeaderMap;
use portal_network::client_address;

use super::main_file::MainFile;

pub const FORWARDING_HEADERS: [&str; 3] = ["forwarded", "x-forwarded-for", "x-real-ip"];

pub fn sees_details(peer: Option<SocketAddr>, headers: &HeaderMap, main: &MainFile) -> bool {
    let Some(peer) = peer else {
        return false;
    };
    let from = peer.ip().to_canonical();
    let through_trusted = main.trusted.iter().any(|network| network.contains(&from));
    let forwarded = FORWARDING_HEADERS
        .iter()
        .any(|name| headers.contains_key(*name));
    if forwarded && !through_trusted {
        return false;
    }
    let visitor = client_address(Some(peer), headers, &main.trusted);
    if visitor.is_loopback() {
        return true;
    }
    main.environments
        .as_ref()
        .is_some_and(|environments| !environments.of(visitor).is_internet())
}
