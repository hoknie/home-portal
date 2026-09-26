use std::net::{IpAddr, SocketAddr};

use axum::http::{HeaderMap, HeaderValue};
use ipnet::IpNet;

use super::{client_address, host_interfaces};

fn forwarded(value: &'static str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("x-forwarded-for", HeaderValue::from_static(value));
    headers
}

fn peer(text: &str) -> Option<SocketAddr> {
    Some(text.parse().unwrap())
}

#[test]
fn an_ipv4_address_written_as_ipv6_counts_as_ipv4() {
    let trusted = vec!["127.0.0.1/32".parse().unwrap()];
    assert_eq!(
        client_address(peer("[::ffff:192.168.1.40]:5000"), &HeaderMap::new(), &[]).to_string(),
        "192.168.1.40"
    );
    assert_eq!(
        client_address(
            peer("[::ffff:127.0.0.1]:5000"),
            &forwarded("::ffff:10.8.0.5"),
            &trusted
        )
        .to_string(),
        "10.8.0.5"
    );
}

#[test]
fn a_forwarded_address_from_an_untrusted_peer_is_ignored() {
    let address = client_address(peer("192.168.1.50:5000"), &forwarded("10.0.0.9"), &[]);
    assert_eq!(address, "192.168.1.50".parse::<IpAddr>().unwrap());
}

#[test]
fn a_trusted_proxy_passes_on_the_last_untrusted_hop() {
    let trusted: Vec<IpNet> = vec!["172.16.0.0/12".parse().unwrap()];
    let address = client_address(
        peer("172.17.0.2:5000"),
        &forwarded("203.0.113.7, 10.0.0.9, 172.17.0.3"),
        &trusted,
    );
    assert_eq!(address, "10.0.0.9".parse::<IpAddr>().unwrap());
}

#[test]
fn a_trusted_proxy_without_the_header_is_the_client() {
    let trusted: Vec<IpNet> = vec!["127.0.0.1/32".parse().unwrap()];
    let address = client_address(peer("127.0.0.1:5000"), &HeaderMap::new(), &trusted);
    assert_eq!(address, "127.0.0.1".parse::<IpAddr>().unwrap());
}

#[test]
fn the_host_has_at_least_a_loopback_interface() {
    let interfaces = host_interfaces();
    assert!(interfaces.iter().any(|interface| {
        interface
            .addresses
            .iter()
            .any(|address| address == "127.0.0.1" || address == "::1")
    }));
}
