use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;

use super::support::{CADDY, OUTSIDE, authorize, file, get, json, location, portal, send};
use crate::fakes::{Caddy, Sessions};

#[tokio::test]
async fn a_disabled_proxy_reports_no_routes_and_refuses_to_apply() {
    let portal = portal("");
    let body = json(send(&portal, get("/api/proxy"), CADDY, "local").await).await;
    assert_eq!(body["enabled"], false);
    assert_eq!(body["routes"], serde_json::json!([]));
    let apply = Request::post("/api/proxy/apply")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        send(&portal, apply, CADDY, "local").await.status(),
        StatusCode::CONFLICT
    );
}

#[tokio::test]
async fn applying_while_caddy_is_down_answers_502_and_records_the_error() {
    let portal = portal(&file("http://127.0.0.1:9"));
    let apply = Request::post("/api/proxy/apply")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        send(&portal, apply, CADDY, "local").await.status(),
        StatusCode::BAD_GATEWAY
    );
    let body = json(send(&portal, get("/api/proxy"), CADDY, "local").await).await;
    assert_eq!(body["reachable"], false);
    assert_eq!(body["in_sync"], false);
    assert!(
        body["last_error"]
            .as_str()
            .unwrap()
            .contains("cannot be reached")
    );
}

#[tokio::test]
async fn applying_by_hand_loads_caddy_and_lists_every_route_portal_first() {
    let (caddy, url) = Caddy::on_tcp().await;
    let portal = portal(&file(&url));
    let apply = Request::post("/api/proxy/apply")
        .body(Body::empty())
        .unwrap();
    let response = send(&portal, apply, CADDY, "local").await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = json(response).await;
    assert_eq!(caddy.with(|recorded| recorded.loads), 1);
    assert_eq!(body["reachable"], true);
    assert_eq!(body["in_sync"], true);
    assert!(body["last_applied_at"].is_string());
    assert_eq!(
        body["routes"][0],
        serde_json::json!({"host":"portal.example.com","address":"https://portal.example.com","service":null,"upstream":"http://127.0.0.1:8080","tls":"acme","auth":[],"environments":[]})
    );
    assert_eq!(
        body["routes"][1],
        serde_json::json!({"host":"nas.example.com","address":"https://nas.example.com","service":"nas","upstream":"http://192.168.1.5","tls":"acme","auth":["internet"],"environments":["internet"]})
    );
    assert_eq!(body["routes"][2]["tls"], "internal");
}

#[tokio::test]
async fn from_outside_without_a_session_a_reader_is_sent_to_sign_in() {
    let portal = portal(&file("http://127.0.0.1:9"));
    let response = send(
        &portal,
        authorize("nas.example.com", None, "GET"),
        CADDY,
        "internet",
    )
    .await;
    assert_eq!(response.status(), StatusCode::FOUND);
    assert_eq!(
        location(&response),
        "https://portal.example.com/login/?return=https%3A%2F%2Fnas.example.com%2Fphotos"
    );
    let posted = send(
        &portal,
        authorize("nas.example.com", None, "POST"),
        CADDY,
        "internet",
    )
    .await;
    assert_eq!(posted.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn from_outside_with_a_session_the_request_passes_with_the_users_name() {
    let portal = portal(&file("http://127.0.0.1:9"));
    let request = authorize("nas.example.com", Some(Sessions::COOKIE), "GET");
    let response = send(&portal, request, CADDY, "internet").await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["x-portal-user"], Sessions::USER);
}

#[tokio::test]
async fn from_home_the_request_passes_without_a_user() {
    let portal = portal(&file("http://127.0.0.1:9"));
    let response = send(
        &portal,
        authorize("nas.example.com", None, "GET"),
        CADDY,
        "local",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().get("x-portal-user").is_none());
}

#[tokio::test]
async fn asked_directly_from_an_untrusted_peer_the_endpoint_does_not_exist() {
    let portal = portal(&file("http://127.0.0.1:9"));
    let request = authorize("nas.example.com", Some(Sessions::COOKIE), "GET");
    let response = send(&portal, request, OUTSIDE, "local").await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn an_unknown_forwarded_host_is_not_found() {
    let portal = portal(&file("http://127.0.0.1:9"));
    let response = send(
        &portal,
        authorize("other.example.com", None, "GET"),
        CADDY,
        "internet",
    )
    .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn the_environment_cookie_does_not_weaken_sign_in() {
    let portal = portal(&file("http://127.0.0.1:9"));
    let request = authorize("nas.example.com", Some("portal_environment=local"), "GET");
    let response = send(&portal, request, CADDY, "internet").await;
    assert_eq!(response.status(), StatusCode::FOUND);
}

#[tokio::test]
async fn continuing_goes_only_to_a_published_host() {
    let portal = portal(&file("http://127.0.0.1:9"));
    let known = send(
        &portal,
        get("/api/proxy/continue?to=https%3A%2F%2Fnas.example.com%2Fphotos"),
        OUTSIDE,
        "internet",
    )
    .await;
    assert_eq!(known.status(), StatusCode::SEE_OTHER);
    assert_eq!(location(&known), "https://nas.example.com/photos");
    let evil = send(
        &portal,
        get("/api/proxy/continue?to=https%3A%2F%2Fevil.example.net%2F"),
        OUTSIDE,
        "internet",
    )
    .await;
    assert_eq!(location(&evil), "/");
}

#[tokio::test]
async fn the_root_certificate_is_given_inside_or_with_a_session() {
    let (_caddy, url) = Caddy::on_tcp().await;
    let portal = portal(&file(&url));
    let inside = send(
        &portal,
        get("/api/proxy/root-certificate"),
        OUTSIDE,
        "local",
    )
    .await;
    assert_eq!(inside.status(), StatusCode::OK);
    assert_eq!(inside.headers()["content-type"], "application/x-pem-file");
    let body = inside.into_body().collect().await.unwrap().to_bytes();
    assert!(body.starts_with(b"-----BEGIN CERTIFICATE-----"));
    let outside = send(
        &portal,
        get("/api/proxy/root-certificate"),
        OUTSIDE,
        "internet",
    )
    .await;
    assert_eq!(outside.status(), StatusCode::NOT_FOUND);
    let mut signed_in = get("/api/proxy/root-certificate");
    signed_in
        .headers_mut()
        .insert("cookie", Sessions::COOKIE.parse().unwrap());
    assert_eq!(
        send(&portal, signed_in, OUTSIDE, "internet").await.status(),
        StatusCode::OK
    );
}

#[tokio::test]
async fn without_an_internal_host_or_a_reachable_caddy_there_is_no_root_certificate() {
    let (_caddy, url) = Caddy::on_tcp().await;
    let without = portal(&file(&url).replace(", tls = { mode = \"internal\" }", ""));
    let response = send(
        &without,
        get("/api/proxy/root-certificate"),
        OUTSIDE,
        "local",
    )
    .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let down = portal(&file("http://127.0.0.1:9"));
    let response = send(&down, get("/api/proxy/root-certificate"), OUTSIDE, "local").await;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn another_https_port_is_kept_in_the_sign_in_redirect() {
    let portal = portal(
        &file("http://127.0.0.1:9").replace("enabled = true", "enabled = true\nhttps_port = 8443"),
    );
    let response = send(
        &portal,
        authorize("nas.example.com:8443", None, "GET"),
        CADDY,
        "internet",
    )
    .await;
    assert_eq!(response.status(), StatusCode::FOUND);
    assert_eq!(
        location(&response),
        "https://portal.example.com:8443/login/?return=https%3A%2F%2Fnas.example.com%3A8443%2Fphotos"
    );
}

#[tokio::test]
async fn without_a_caddy_table_the_download_comes_from_the_latest_github_release() {
    let portal = portal("");
    let body = json(send(&portal, get("/api/proxy"), CADDY, "local").await).await;
    let caddy = &body["caddy"];
    assert_eq!(caddy["source"], crate::types::CaddySource::DEFAULT_BASE);
    assert_eq!(caddy["version"], "latest");
    assert_eq!(
        caddy["release_url"],
        "https://api.github.com/repos/caddyserver/caddy/releases/latest"
    );
    assert_eq!(
        caddy["platform"],
        crate::types::Platform::current()
            .map(|platform| platform.name())
            .ok()
            .map_or(serde_json::Value::Null, serde_json::Value::from)
    );
    assert_eq!(caddy["installed_from"], serde_json::Value::Null);
}
