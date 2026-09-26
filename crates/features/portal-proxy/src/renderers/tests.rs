use std::net::SocketAddr;

use portal_model::{Publication, TlsMode, TlsPolicy};
use serde_json::{Value, json};

use super::render;
use crate::types::{AdminAddress, ProxySettings, PublishedService};

const ADAPTED: &str = include_str!("../../fixtures/caddy/adapted.json");

fn settings() -> ProxySettings {
    ProxySettings {
        enabled: true,
        managed: false,
        http_port: 80,
        https_port: 443,
        admin: AdminAddress::default(),
        portal_host: Some("portal.example.com".into()),
        cookie_domain: Some("example.com".into()),
        tls: TlsPolicy::default(),
        caddy: Default::default(),
        doh_host: None,
    }
}

fn portal() -> SocketAddr {
    "127.0.0.1:8080".parse().unwrap()
}

fn service(id: &str, host: &str, upstream: &str) -> PublishedService {
    PublishedService {
        id: id.into(),
        upstream: upstream.into(),
        publication: Publication::new(host),
    }
}

fn routes(rendered: &Value) -> &Vec<Value> {
    rendered["apps"]["http"]["servers"]["home-portal"]["routes"]
        .as_array()
        .unwrap()
}

fn handlers_of<'a>(rendered: &'a Value, host: &str) -> &'a Vec<Value> {
    routes(rendered)
        .iter()
        .find(|route| route["match"][0]["host"][0] == host)
        .map(|route| {
            route["handle"][0]["routes"][0]["handle"]
                .as_array()
                .unwrap()
        })
        .unwrap()
}

fn adapted_handlers(host: &str) -> Vec<Value> {
    let adapted: Value = serde_json::from_str(ADAPTED).unwrap();
    adapted["apps"]["http"]["servers"]["srv0"]["routes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|route| route["match"][0]["host"][0] == host)
        .map(|route| {
            route["handle"][0]["routes"][0]["handle"]
                .as_array()
                .unwrap()
                .clone()
        })
        .unwrap()
}

#[test]
fn the_portal_alone_renders_to_one_complete_document() {
    let rendered = render(&settings(), &[], portal());
    assert_eq!(
        rendered,
        json!({
            "admin": { "listen": "127.0.0.1:2019" },
            "apps": {
                "http": { "http_port": 80, "https_port": 443, "servers": { "home-portal": {
                    "listen": [":443"],
                    "routes": [{
                        "match": [{ "host": ["portal.example.com"] }],
                        "handle": [{ "handler": "subroute", "routes": [{ "handle": [
                            { "handler": "headers", "request": { "delete": ["X-Portal-User"] } },
                            { "handler": "reverse_proxy", "upstreams": [{ "dial": "127.0.0.1:8080" }] }
                        ] }] }],
                        "terminal": true
                    }]
                } } },
                "tls": { "automation": { "policies": [
                    { "subjects": ["portal.example.com"], "issuers": [{ "module": "acme" }] }
                ] } }
            }
        })
    );
}

#[test]
fn rendering_the_same_configuration_twice_is_byte_identical() {
    let services = [service(
        "media",
        "media.example.com",
        "http://192.168.1.10:8096",
    )];
    let first = serde_json::to_string_pretty(&render(&settings(), &services, portal())).unwrap();
    let second = serde_json::to_string_pretty(&render(&settings(), &services, portal())).unwrap();
    assert_eq!(first, second);
}

#[test]
fn a_service_behind_sign_in_asks_the_portal_exactly_as_caddys_forward_auth_does() {
    let mut nas = service("nas", "nas.example.com", "https://192.168.1.5:5001");
    nas.publication.auth = vec!["internet".into()];
    nas.publication.upstream_verify = false;
    let rendered = render(&settings(), &[nas], portal());
    let adapted = adapted_handlers("nas.example.com");
    let handlers = handlers_of(&rendered, "nas.example.com");
    assert_eq!(handlers[1], adapted[0]);
    assert_eq!(handlers[2], adapted[1]);
}

#[test]
fn every_host_drops_the_user_header_the_visitor_sent_before_anything_else() {
    let rendered = render(
        &settings(),
        &[service(
            "media",
            "media.example.com",
            "http://192.168.1.10:8096",
        )],
        portal(),
    );
    let deletion = &adapted_handlers("media.home.arpa")[0];
    assert_eq!(&handlers_of(&rendered, "portal.example.com")[0], deletion);
    assert_eq!(&handlers_of(&rendered, "media.example.com")[0], deletion);
}

#[test]
fn a_service_without_sign_in_goes_straight_to_its_upstream() {
    let rendered = render(
        &settings(),
        &[service(
            "media",
            "media.example.com",
            "http://192.168.1.10:8096",
        )],
        portal(),
    );
    assert_eq!(
        handlers_of(&rendered, "media.example.com")[1],
        json!({ "handler": "reverse_proxy", "upstreams": [{ "dial": "192.168.1.10:8096" }] })
    );
    assert_eq!(handlers_of(&rendered, "media.example.com").len(), 2);
}

#[test]
fn an_https_upstream_is_verified_unless_told_otherwise() {
    let verified = service("nas", "nas.example.com", "https://nas.lan");
    let rendered = render(&settings(), &[verified], portal());
    assert_eq!(
        handlers_of(&rendered, "nas.example.com")[1]["transport"],
        json!({ "protocol": "http", "tls": {} })
    );
    assert_eq!(
        handlers_of(&rendered, "nas.example.com")[1]["upstreams"][0]["dial"],
        "nas.lan:443"
    );
}

#[test]
fn each_tls_mode_is_rendered_as_caddy_would_adapt_it() {
    let mut media = service("media", "media.home.arpa", "http://192.168.1.10:8096");
    media.publication.tls = Some(TlsPolicy {
        mode: TlsMode::Internal,
        ..TlsPolicy::default()
    });
    let mut files = service("files", "files.example.com", "http://192.168.1.20");
    files.publication.tls = Some(TlsPolicy {
        mode: TlsMode::Files,
        certificate: Some("/etc/ssl/files.pem".into()),
        key: Some("/etc/ssl/files.key".into()),
        ..TlsPolicy::default()
    });
    let mut settings = settings();
    settings.tls.email = Some("owner@example.com".into());
    let rendered = render(&settings, &[media, files], portal());
    let adapted: Value = serde_json::from_str(ADAPTED).unwrap();
    let tls = &rendered["apps"]["tls"];
    assert_eq!(
        tls["certificates"]["load_files"][0]["certificate"],
        adapted["apps"]["tls"]["certificates"]["load_files"][0]["certificate"]
    );
    assert_eq!(
        tls["automation"]["policies"],
        json!([
            { "subjects": ["portal.example.com"], "issuers": [{ "module": "acme", "email": "owner@example.com" }] },
            { "subjects": ["media.home.arpa"], "issuers": adapted["apps"]["tls"]["automation"]["policies"][1]["issuers"] }
        ])
    );
    let selections = &rendered["apps"]["http"]["servers"]["home-portal"]["tls_connection_policies"];
    assert_eq!(
        selections[0]["match"],
        adapted["apps"]["http"]["servers"]["srv0"]["tls_connection_policies"][0]["match"]
    );
    assert_eq!(
        selections[0]["certificate_selection"]["any_tag"][0],
        tls["certificates"]["load_files"][0]["tags"][0]
    );
    assert_eq!(selections[1], json!({}));
}

#[test]
fn a_portal_listening_everywhere_is_reached_on_loopback() {
    let rendered = render(&settings(), &[], "0.0.0.0:8080".parse().unwrap());
    assert_eq!(
        handlers_of(&rendered, "portal.example.com")[1]["upstreams"][0]["dial"],
        "127.0.0.1:8080"
    );
    let rendered = render(&settings(), &[], "[::]:8080".parse().unwrap());
    assert_eq!(
        handlers_of(&rendered, "portal.example.com")[1]["upstreams"][0]["dial"],
        "[::1]:8080"
    );
}

#[test]
fn a_unix_admin_socket_is_kept_in_the_document() {
    let mut settings = settings();
    settings.admin = AdminAddress::parse("unix:/run/caddy/admin.sock").unwrap();
    assert_eq!(
        render(&settings, &[], portal())["admin"]["listen"],
        "unix//run/caddy/admin.sock"
    );
}

#[test]
fn caddy_never_installs_its_root_into_the_system_trust_store() {
    let mut media = service("media", "media.home.arpa", "http://192.168.1.10:8096");
    media.publication.tls = Some(TlsPolicy {
        mode: TlsMode::Internal,
        ..TlsPolicy::default()
    });
    let rendered = render(&settings(), &[media], portal());
    assert_eq!(
        rendered["apps"]["pki"],
        json!({ "certificate_authorities": { "local": { "install_trust": false } } })
    );
    assert!(
        render(&settings(), &[], portal())["apps"]
            .get("pki")
            .is_none()
    );
}

#[test]
fn caddy_listens_on_the_configured_ports() {
    let mut settings = settings();
    settings.http_port = 8080;
    settings.https_port = 8443;
    let rendered = render(&settings, &[], portal());
    assert_eq!(rendered["apps"]["http"]["http_port"], 8080);
    assert_eq!(rendered["apps"]["http"]["https_port"], 8443);
    assert_eq!(
        rendered["apps"]["http"]["servers"]["home-portal"]["listen"],
        json!([":8443"])
    );
}

#[test]
fn dns_over_https_on_its_own_host_is_routed_to_the_portal_before_anything_else_for_that_host() {
    assert!(
        routes(&render(&settings(), &[], portal()))
            .iter()
            .all(|route| route["match"][0]["path"].is_null())
    );
    let with_doh = ProxySettings {
        doh_host: Some("dns.example.com".into()),
        ..settings()
    };
    let rendered = render(&with_doh, &[], portal());
    let route = routes(&rendered)
        .iter()
        .find(|route| route["match"][0]["host"][0] == "dns.example.com")
        .unwrap();
    assert_eq!(route["match"][0]["path"], json!(["/dns-query"]));
    assert_eq!(route["handle"][0]["handler"], "reverse_proxy");
    assert_eq!(route["handle"][0]["upstreams"][0]["dial"], "127.0.0.1:8080");
    let policies = rendered["apps"]["tls"]["automation"]["policies"].to_string();
    assert!(policies.contains("dns.example.com"), "{policies}");
}
