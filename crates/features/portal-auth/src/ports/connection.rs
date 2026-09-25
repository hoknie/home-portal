use std::net::{IpAddr, SocketAddr};

use axum::http::HeaderMap;

use crate::types::CookieScope;

pub trait Connection: Send + Sync {
    fn client_address(&self, peer: Option<SocketAddr>, headers: &HeaderMap) -> IpAddr;

    fn cookie_scope(&self, peer: Option<SocketAddr>, headers: &HeaderMap) -> CookieScope;
}
