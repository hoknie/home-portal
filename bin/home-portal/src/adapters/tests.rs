use std::fs;
use std::sync::Arc;

use portal_config::ConfigStore;
use portal_model::Environment;
use portal_proxy::PublishedServices;
use portal_services::{Publishing, ServicesFeature};

use super::{ProxyPublishing, ServicePublications};

const FILE: &str = "[environments.local]\nnetworks = [\"192.168.0.0/16\"]\n\n[[services]]\nid = \"media\"\nname = \"Media\"\nurl = \"http://media.lan:8096\"\naddresses = { local = \"http://192.168.1.10:8096\" }\nproxy = { host = \"media.example.com\" }\nprobe = { enabled = false }\n\n[[services]]\nid = \"nas\"\nname = \"NAS\"\nurl = \"http://192.168.1.5\"\nproxy = { host = \"nas.example.com\", upstream = \"https://192.168.1.5:5001\" }\nprobe = { enabled = false }\n\n[[services]]\nid = \"printer\"\nname = \"Printer\"\nurl = \"http://192.168.1.30\"\nprobe = { enabled = false }\n";

fn services(text: &str) -> (tempfile::TempDir, Arc<ConfigStore>, Arc<ServicesFeature>) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    fs::write(&path, text).unwrap();
    let store = Arc::new(ConfigStore::open(&path).unwrap());
    let services = ServicesFeature::new(
        store.clone(),
        Environment::parse("local").unwrap(),
        Vec::new(),
        Arc::new(ProxyPublishing {
            configuration: store.clone(),
        }),
    )
    .unwrap();
    (directory, store, Arc::new(services))
}

#[test]
fn a_service_without_an_upstream_is_proxied_to_the_address_it_is_probed_at() {
    let (_directory, _, services) = services(FILE);
    let published = ServicePublications { services }.published();
    let upstreams: Vec<(&str, &str)> = published
        .iter()
        .map(|service| (service.id.as_str(), service.upstream.as_str()))
        .collect();
    assert_eq!(
        upstreams,
        vec![
            ("media", "http://192.168.1.10:8096"),
            ("nas", "https://192.168.1.5:5001")
        ]
    );
}

#[test]
fn publishing_follows_the_proxy_section() {
    let (_directory, store, _) = services(FILE);
    let publishing = ProxyPublishing {
        configuration: store,
    };
    assert_eq!(publishing.https_port(), None);
    let (_directory, store, _) = services(&format!(
        "[network]\ntrusted_proxies = [\"127.0.0.1\"]\n\n[proxy]\nenabled = true\nportal_host = \"portal.example.com\"\n\n{FILE}"
    ));
    assert_eq!(
        ProxyPublishing {
            configuration: store
        }
        .https_port(),
        Some(443)
    );
}

fn connection(text: &str) -> (tempfile::TempDir, super::NetworkConnection) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    fs::write(&path, text).unwrap();
    let configuration = Arc::new(ConfigStore::open(&path).unwrap());
    (directory, super::NetworkConnection { configuration })
}

const PROXIED: &str = "[network]\ntrusted_proxies = [\"127.0.0.1\"]\n\n[proxy]\nenabled = true\nportal_host = \"portal.example.com\"\ncookie_domain = \"example.com\"\n";

fn through(host: &str) -> axum::http::HeaderMap {
    let mut headers = axum::http::HeaderMap::new();
    headers.insert("x-forwarded-host", host.parse().unwrap());
    headers
}

fn peer(address: &str) -> Option<std::net::SocketAddr> {
    Some(format!("{address}:40000").parse().unwrap())
}

#[test]
fn one_sign_in_through_caddy_covers_every_published_host_under_the_cookie_domain() {
    use portal_auth::Connection;
    let (_directory, connection) = connection(PROXIED);
    let scope = connection.cookie_scope(peer("127.0.0.1"), &through("portal.example.com:443"));
    assert_eq!(scope.domain.as_deref(), Some("example.com"));
    assert!(scope.secure);
}

#[test]
fn a_sign_in_made_directly_while_the_proxy_is_on_keeps_a_plain_cookie() {
    use portal_auth::Connection;
    let (_directory, connection) = connection(PROXIED);
    let direct = connection.cookie_scope(peer("127.0.0.1"), &axum::http::HeaderMap::new());
    assert_eq!(direct, portal_auth::CookieScope::default());
    let untrusted = connection.cookie_scope(peer("192.168.1.40"), &through("portal.example.com"));
    assert_eq!(untrusted, portal_auth::CookieScope::default());
    let elsewhere = connection.cookie_scope(peer("127.0.0.1"), &through("portal.example.org"));
    assert_eq!(elsewhere, portal_auth::CookieScope::default());
}

#[test]
fn without_the_proxy_the_cookie_has_no_domain() {
    use portal_auth::Connection;
    let (_directory, connection) =
        connection(&PROXIED.replace("enabled = true", "enabled = false"));
    let scope = connection.cookie_scope(peer("127.0.0.1"), &through("portal.example.com"));
    assert_eq!(scope, portal_auth::CookieScope::default());
}

#[test]
fn only_a_trusted_proxy_is_trusted_as_a_peer() {
    use portal_proxy::TrustedPeers;
    let (_directory, connection) = connection(PROXIED);
    assert!(connection.trusts("127.0.0.1".parse().unwrap()));
    assert!(!connection.trusts("192.168.1.40".parse().unwrap()));
}
