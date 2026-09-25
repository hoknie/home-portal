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

#[tokio::test]
async fn enabling_from_the_interface_writes_the_section_and_trusts_loopback() {
    let portal = portal("[network]\nport = 8080\n");
    let revision = revision_of(&portal).await;
    let response = send(&portal, put(ENABLE, Some(&revision)), CADDY, "local").await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = json(response).await;
    assert_eq!(body["enabled"], true);
    assert_eq!(body["settings"]["portal_host"], "portal.example.com");
    assert_eq!(body["settings"]["tls"]["mode"], "internal");
    let text = fs::read_to_string(&portal.path).unwrap();
    assert!(text.contains("trusted_proxies = [\"127.0.0.1\"]"), "{text}");
    assert!(text.contains("[proxy]\nenabled = true"), "{text}");
}

#[tokio::test]
async fn disabling_keeps_the_rest_of_the_section() {
    let portal = portal("[network]\ntrusted_proxies = [\"127.0.0.1\"]\n");
    let revision = revision_of(&portal).await;
    send(&portal, put(ENABLE, Some(&revision)), CADDY, "local").await;
    let revision = revision_of(&portal).await;
    let disable = ENABLE.replace("\"enabled\":true", "\"enabled\":false");
    let response = send(&portal, put(&disable, Some(&revision)), CADDY, "local").await;
    assert_eq!(json(response).await["enabled"], false);
    let text = fs::read_to_string(&portal.path).unwrap();
    assert!(
        text.contains("enabled = false") && text.contains("portal.example.com"),
        "{text}"
    );
}

#[tokio::test]
async fn enabling_without_the_portal_host_is_refused_on_its_field() {
    let portal = portal("");
    let revision = revision_of(&portal).await;
    let response = send(
        &portal,
        put(r#"{"enabled":true}"#, Some(&revision)),
        CADDY,
        "local",
    )
    .await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let fields: Vec<String> = json(response).await["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|error| error["field"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(fields, vec!["proxy.portal_host"]);
    assert_eq!(fs::read_to_string(&portal.path).unwrap(), "");
}

#[tokio::test]
async fn a_change_needs_the_revision() {
    let portal = portal("");
    let response = send(&portal, put(ENABLE, None), CADDY, "local").await;
    assert_eq!(response.status(), StatusCode::PRECONDITION_REQUIRED);
}

#[tokio::test]
async fn other_ports_are_written_and_shown_in_every_address() {
    let portal = portal("[network]\ntrusted_proxies = [\"127.0.0.1\"]\n");
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
    let text = fs::read_to_string(&portal.path).unwrap();
    assert!(
        text.contains("http_port = 8080\nhttps_port = 8443"),
        "{text}"
    );
    let revision = revision_of(&portal).await;
    let back = send(&portal, put(ENABLE, Some(&revision)), CADDY, "local").await;
    assert_eq!(back.status(), StatusCode::OK);
    assert!(!fs::read_to_string(&portal.path).unwrap().contains("_port"));
}

#[tokio::test]
async fn one_port_for_both_is_refused() {
    let portal = portal("[network]\ntrusted_proxies = [\"127.0.0.1\"]\n");
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
