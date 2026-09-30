use std::fs;
use std::net::{TcpListener, UdpSocket};
use std::time::{Duration, Instant};

use axum::Router;
use axum::body::Body;
use axum::http::header::{CONTENT_TYPE, COOKIE, ETAG, IF_MATCH, SET_COOKIE};
use axum::http::{Request, Response, StatusCode};
use home_portal::{adopt, assemble, registered};
use http_body_util::BodyExt;
use portal_auth::hash_password;
use serde_json::Value;
use tower::ServiceExt;

#[path = "support/portal.rs"]
mod portal;

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn configuration(port: u16) -> String {
    let hash = hash_password("secret").unwrap();
    format!(
        "# parts\n[modules]\nproxy = true\ndns = true\n\n[network]\ntrusted_proxies = [\"127.0.0.1\"]\n\n[environments.local]\nnetworks = [\"127.0.0.0/8\"]\n\n[proxy]\nportal_host = \"portal.home\"\nadmin = \"http://127.0.0.1:{}\"\n\n[dns]\naddress = \"127.0.0.1\"\nport = {port}\nzones = [\"home\"]\n\n[[users]]\nname = \"admin\"\npassword_hash = \"{hash}\"\ngroup = \"admin\"\n\n[permissions]\nrequest_at_start = false\n",
        free_port()
    )
}

async fn body_of(response: Response<Body>) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
}

async fn signed_in(portal: &Router) -> String {
    let response = portal
        .clone()
        .oneshot(
            Request::post("/api/session")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"name":"admin","password":"secret"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    response.headers()[SET_COOKIE]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string()
}

fn listening(port: u16) -> bool {
    UdpSocket::bind(("127.0.0.1", port)).is_err()
}

async fn eventually(check: impl Fn() -> bool, within: Duration) -> bool {
    let began = Instant::now();
    while began.elapsed() < within {
        if check() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    check()
}

#[tokio::test]
async fn users_need_a_session() {
    let directory = tempfile::tempdir().unwrap();
    let path = portal::with_extra(&directory, "secret", "");
    let registry = registered(&portal::wiring_for(&path)).unwrap();
    let response = assemble(&registry)
        .oneshot(Request::get("/api/users").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn modules_need_a_session() {
    let directory = tempfile::tempdir().unwrap();
    let path = portal::with_extra(&directory, "secret", "");
    let registry = registered(&portal::wiring_for(&path)).unwrap();
    let response = assemble(&registry)
        .oneshot(Request::get("/api/modules").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test(flavor = "multi_thread")]
async fn switching_dns_off_through_the_api_closes_its_port_within_five_seconds() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    let port = free_port();
    fs::write(&path, configuration(port)).unwrap();
    let wiring = portal::wiring_for(&path);
    let registry = registered(&wiring).unwrap();
    adopt(&wiring.configuration, &registry).unwrap();
    for feature in &registry.features {
        for task in feature.loops() {
            tokio::spawn(task);
        }
    }
    let portal = assemble(&registry);
    assert!(eventually(|| listening(port), Duration::from_secs(5)).await);
    let cookie = signed_in(&portal).await;
    let shown = portal
        .clone()
        .oneshot(
            Request::get("/api/modules")
                .header(COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let revision = shown.headers()[ETAG].to_str().unwrap().to_string();
    let switched = portal
        .clone()
        .oneshot(
            Request::put("/api/modules/dns")
                .header(COOKIE, &cookie)
                .header(CONTENT_TYPE, "application/json")
                .header(IF_MATCH, revision)
                .body(Body::from(r#"{"enabled":false}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(switched.status(), StatusCode::OK);
    let body = body_of(switched).await;
    assert_eq!(body["modules"][1]["name"], "dns");
    assert_eq!(body["modules"][1]["enabled"], false);
    assert!(eventually(|| !listening(port), Duration::from_secs(5)).await);
    let text = fs::read_to_string(&path).unwrap();
    assert!(
        text.starts_with("# parts\n[modules]\nproxy = true\ndns = false\n"),
        "{text}"
    );
}
