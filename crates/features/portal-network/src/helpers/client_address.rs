use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::http::HeaderMap;
use ipnet::IpNet;

pub const FORWARDED_FOR: &str = "x-forwarded-for";

pub fn client_address(peer: Option<SocketAddr>, headers: &HeaderMap, trusted: &[IpNet]) -> IpAddr {
    let peer = peer.map_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED), |peer| peer.ip());
    let is_trusted = |address: &IpAddr| trusted.iter().any(|network| network.contains(address));
    if !is_trusted(&peer) {
        return peer;
    }
    let hops: Vec<IpAddr> = headers
        .get_all(FORWARDED_FOR)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .filter_map(|hop| hop.trim().parse().ok())
        .collect();
    hops.into_iter()
        .rev()
        .find(|hop| !is_trusted(hop))
        .unwrap_or(peer)
}
