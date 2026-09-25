mod support;

use std::net::SocketAddr;

use axum::Router;
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::header::COOKIE;
use axum::http::{Request, StatusCode};
use home_portal::{assemble, registered};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

const PORTAL: &str = r#"
[environments.local]
networks = ["192.168.0.0/16"]

[[services]]
id = "blog"
name = "Blog"
url = "https://blog.example.com"
public = true
public_status = true
links = [{ title = "Editor", url = "https://blog.example.com/wp-admin-hidden" }]
notes = "The editor password is in the vault under blog-editor-note"
widgets = ["box"]
probe = { enabled = false }

[[services]]
id = "nas"
name = "NAS"
url = "http://nas.local"
public = true
environments = ["local"]
probe = { enabled = false }

[[services]]
id = "secret-box"
name = "Secret box"
url = "http://secret.local"
probe = { enabled = false }

[[dashboard.sections]]
id = "now"
title = "Right now"

[[dashboard.sections]]
id = "shelf"
title = "Private shelf"

[[dashboard.widgets]]
type = "host-metrics"
id = "box"
public = true
size = "half"

[[dashboard.widgets]]
type = "host-metrics"
id = "private-box"
section = "shelf"
"#;

fn portal() -> Router {
    let directory = Box::leak(Box::new(tempfile::tempdir().unwrap()));
    let path = support::with_extra(directory, "secret", PORTAL);
    let text = std::fs::read_to_string(&path).unwrap().replace(
        "trusted_proxies = []",
        r#"trusted_proxies = ["127.0.0.0/8"]"#,
    );
    std::fs::write(&path, text).unwrap();
    assemble(&registered(&support::wiring_for(&path)).unwrap())
}

async fn ask(path: &str, forwarded: Option<&str>) -> (StatusCode, Value) {
    ask_choosing(path, forwarded, None).await
}

async fn ask_choosing(
    path: &str,
    forwarded: Option<&str>,
    chosen: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::get(path);
    if let Some(chosen) = chosen {
        builder = builder.header(COOKIE, format!("portal_environment={chosen}"));
    }
    if let Some(forwarded) = forwarded {
        builder = builder.header("x-forwarded-for", forwarded);
    }
    let mut request = builder.body(Body::empty()).unwrap();
    let peer: SocketAddr = "127.0.0.1:5000".parse().unwrap();
    request.extensions_mut().insert(ConnectInfo(peer));
    let response = portal().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = serde_json::from_slice(&bytes)
        .unwrap_or(Value::String(String::from_utf8_lossy(&bytes).to_string()));
    (status, body)
}

fn ids(body: &Value, key: &str) -> Vec<String> {
    body[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["id"].as_str().unwrap_or_default().to_string())
        .collect()
}

#[tokio::test]
async fn the_public_portal_answers_without_a_session() {
    let (status, body) = ask("/api/public/portal", Some("192.168.1.40")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["environment"], "local");
    assert_eq!(ids(&body, "services"), vec!["blog", "nas"]);
    assert_eq!(ids(&body, "widgets"), vec!["box"]);
}

#[tokio::test]
async fn a_visitor_from_outside_sees_only_what_is_public_there() {
    let (_, body) = ask("/api/public/portal", Some("203.0.113.5")).await;
    assert_eq!(body["environment"], "internet");
    assert_eq!(ids(&body, "services"), vec!["blog"]);
}

#[tokio::test]
async fn a_status_is_shown_only_where_the_owner_asked_for_it() {
    let (_, body) = ask("/api/public/portal", Some("192.168.1.40")).await;
    let services = body["services"].as_array().unwrap();
    assert!(
        !services[0]["status"].is_null(),
        "blog carries public_status"
    );
    assert!(services[1]["status"].is_null(), "nas does not");
}

#[tokio::test]
async fn a_widget_that_is_not_public_answers_as_an_unknown_one() {
    let (hidden, _) = ask("/api/public/widgets/private-box/data", Some("192.168.1.40")).await;
    let (unknown, _) = ask("/api/public/widgets/nope/data", Some("192.168.1.40")).await;
    assert_eq!(hidden, StatusCode::NOT_FOUND);
    assert_eq!(unknown, StatusCode::NOT_FOUND);
    let (public, body) = ask("/api/public/widgets/box/data", Some("192.168.1.40")).await;
    assert_eq!(public, StatusCode::OK);
    assert!(
        body["data"]["memory"]["total_bytes"]
            .as_u64()
            .unwrap_or_default()
            > 0
    );
}

#[tokio::test]
async fn the_private_half_still_needs_a_session() {
    for path in [
        "/api/services",
        "/api/dashboard",
        "/api/secrets",
        "/api/widgets/box/data",
    ] {
        let (status, _) = ask(path, Some("192.168.1.40")).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{path}");
    }
}

#[tokio::test]
async fn a_public_widget_carries_its_section_and_size_and_only_its_section_is_listed() {
    let (_, body) = ask("/api/public/portal", Some("203.0.113.5")).await;
    assert_eq!(
        body["sections"],
        serde_json::json!([{ "id": "now", "title": "Right now" }])
    );
    assert_eq!(body["widgets"][0]["section"], "now");
    assert_eq!(body["widgets"][0]["size"], "half");
}

#[tokio::test]
async fn nothing_private_appears_anywhere_in_the_public_payload() {
    let (_, body) = ask("/api/public/portal", Some("203.0.113.5")).await;
    let text = serde_json::to_string(&body).unwrap();
    for private in [
        "secret-box",
        "Secret box",
        "secret.local",
        "nas.local",
        "admin",
        "argon2",
        "private-box",
        "revision",
        "password",
        "wp-admin-hidden",
        "blog-editor-note",
        "Editor",
        "\"links\"",
        "\"notes\"",
        "Private shelf",
        "shelf",
    ] {
        assert!(
            !text.contains(private),
            "{private} appears in the public portal: {text}"
        );
    }
}

#[tokio::test]
async fn a_visitor_at_home_may_see_the_public_page_as_from_outside() {
    let (_, body) =
        ask_choosing("/api/public/portal", Some("192.168.1.40"), Some("internet")).await;
    assert_eq!(body["environment"], "internet");
    assert_eq!(body["detected"], "local");
    assert_eq!(ids(&body, "services"), vec!["blog"]);
}

#[tokio::test]
async fn a_choice_from_outside_changes_nothing_on_the_public_page() {
    let (_, plain) = ask("/api/public/portal", Some("203.0.113.5")).await;
    let (_, chosen) = ask_choosing("/api/public/portal", Some("203.0.113.5"), Some("local")).await;
    for key in ["environment", "detected", "sections", "widgets"] {
        assert_eq!(plain[key], chosen[key], "{key}");
    }
    assert_eq!(ids(&plain, "services"), ids(&chosen, "services"));
    assert_eq!(chosen["switchable"], false);
    assert!(chosen.get("environments").is_none());
}
