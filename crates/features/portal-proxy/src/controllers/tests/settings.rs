use std::fs;

use axum::body::Body;
use axum::http::{Request, StatusCode};

use super::support::{CADDY, Portal, get, json, portal, send};

async fn revision_of(portal: &Portal) -> String {
    let response = send(portal, get("/api/proxy"), CADDY, "local").await;
    response.headers()["etag"].to_str().unwrap().to_string()
}

fn put(body: &str, revision: Option<&str>) -> Request<Body> {
    let mut builder = Request::put("/api/proxy").header("content-type", "application/json");
    if let Some(revision) = revision {
        builder = builder.header("if-match", revision);
    }
    builder.body(Body::from(body.to_string())).unwrap()
}

const ENABLE: &str = r#"{"enabled":true,"portal_host":"portal.example.com","cookie_domain":"example.com","tls":{"mode":"internal"}}"#;
const SWITCHED_ON: &str = "[modules]\nproxy = true\n\n[network]\ntrusted_proxies = [\"127.0.0.1\"]\n\n[proxy]\nportal_host = \"old.example.com\"\n";

#[tokio::test]
async fn editing_the_host_while_the_proxy_is_on_publishes_it() {
    let portal = portal(SWITCHED_ON);
    let revision = revision_of(&portal).await;
    let response = send(&portal, put(ENABLE, Some(&revision)), CADDY, "local").await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = json(response).await;
    assert_eq!(body["enabled"], true);
    assert_eq!(body["settings"]["portal_host"], "portal.example.com");
    assert_eq!(body["settings"]["tls"]["mode"], "internal");
    let text = fs::read_to_string(&portal.path).unwrap();
    assert!(text.contains("[modules]\nproxy = true"), "{text}");
}

#[tokio::test]
async fn put_proxy_ignores_enabled_and_writes_no_switch() {
    let portal = portal("[network]\nport = 8080\n");
    let revision = revision_of(&portal).await;
    let response = send(&portal, put(ENABLE, Some(&revision)), CADDY, "local").await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = json(response).await;
    assert_eq!(body["enabled"], false);
    assert_eq!(body["settings"]["portal_host"], "portal.example.com");
    let text = fs::read_to_string(&portal.path).unwrap();
    assert!(!text.contains("enabled"), "{text}");
    assert!(!text.contains("trusted_proxies"), "{text}");
    let proxy = fs::read_to_string(&portal.proxy).unwrap();
    assert!(!proxy.contains("enabled"), "{proxy}");
}

#[tokio::test]
async fn a_legacy_enabled_key_still_switches_the_proxy_on() {
    let portal = portal(
        "[modules]\nproxy = true\n\n[network]\ntrusted_proxies = [\"127.0.0.1\"]\n\n[proxy]\nportal_host = \"portal.example.com\"\n",
    );
    let response = send(&portal, get("/api/proxy"), CADDY, "local").await;
    assert_eq!(json(response).await["enabled"], true);
}

#[tokio::test]
async fn the_modules_key_wins_over_the_legacy_key() {
    let portal =
        portal("[modules]\nproxy = false\n\n[proxy]\nportal_host = \"portal.example.com\"\n");
    let response = send(&portal, get("/api/proxy"), CADDY, "local").await;
    let body = json(response).await;
    assert_eq!(body["enabled"], false);
    assert_eq!(body["routes"].as_array().map(Vec::len), Some(0));
}

#[tokio::test]
async fn incomplete_settings_are_accepted_while_the_proxy_is_off() {
    let portal = portal("");
    let revision = revision_of(&portal).await;
    let response = send(
        &portal,
        put(r#"{"cookie_domain":"example.com"}"#, Some(&revision)),
        CADDY,
        "local",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn a_missing_host_is_refused_on_its_field_while_the_proxy_is_on() {
    let portal = portal(SWITCHED_ON);
    let before = fs::read_to_string(&portal.proxy).unwrap();
    let revision = revision_of(&portal).await;
    let response = send(&portal, put("{}", Some(&revision)), CADDY, "local").await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let fields: Vec<String> = json(response).await["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|error| error["field"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(fields, vec!["proxy.portal_host"]);
    assert_eq!(fs::read_to_string(&portal.proxy).unwrap(), before);
}

#[tokio::test]
async fn a_change_needs_the_revision() {
    let portal = portal("");
    let response = send(&portal, put(ENABLE, None), CADDY, "local").await;
    assert_eq!(response.status(), StatusCode::PRECONDITION_REQUIRED);
}

#[tokio::test]
async fn other_ports_are_written_and_shown_in_every_address() {
    let portal = portal(SWITCHED_ON);
    let revision = revision_of(&portal).await;
    let body = ENABLE.replace(
        "{\"enabled\":true",
        "{\"enabled\":true,\"http_port\":8080,\"https_port\":8443",
    );
    let response = send(&portal, put(&body, Some(&revision)), CADDY, "local").await;
    assert_eq!(response.status(), StatusCode::OK);
    let answer = json(response).await;
    assert_eq!(answer["settings"]["https_port"], 8443);
    assert_eq!(
        answer["routes"][0]["address"],
        "https://portal.example.com:8443"
    );
    let text = fs::read_to_string(&portal.proxy).unwrap();
    assert!(
        text.contains("http_port = 8080\nhttps_port = 8443"),
        "{text}"
    );
    let revision = revision_of(&portal).await;
    let back = send(&portal, put(ENABLE, Some(&revision)), CADDY, "local").await;
    assert_eq!(back.status(), StatusCode::OK);
    assert!(!fs::read_to_string(&portal.proxy).unwrap().contains("_port"));
}

#[tokio::test]
async fn one_port_for_both_is_refused() {
    let portal = portal(SWITCHED_ON);
    let revision = revision_of(&portal).await;
    let body = ENABLE.replace(
        "{\"enabled\":true",
        "{\"enabled\":true,\"http_port\":8443,\"https_port\":8443",
    );
    let response = send(&portal, put(&body, Some(&revision)), CADDY, "local").await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        json(response).await["errors"][0]["field"],
        "proxy.https_port"
    );
}
