use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use axum::Router;
use axum::body::Body;
use axum::http::header::{CONTENT_TYPE, COOKIE, RETRY_AFTER, SET_COOKIE};
use axum::http::{HeaderMap, Request, StatusCode};
use axum::middleware::{self, Next};
use axum::response::Response;
use http_body_util::BodyExt;
use portal_config::ConfigStore;
use portal_feature::{EventName, EventSink, Feature, Gate, PortalEvent};
use portal_model::{DetectedEnvironment, Environment, Environments};
use tempfile::TempDir;
use tower::ServiceExt;

use super::AuthFeature;
use crate::helpers::hash_password;
use crate::ports::Connection;
use crate::types::CookieScope;

struct Direct;

impl Connection for Direct {
    fn client_address(&self, _peer: Option<SocketAddr>, _headers: &HeaderMap) -> IpAddr {
        IpAddr::V4(Ipv4Addr::LOCALHOST)
    }

    fn cookie_scope(&self, _peer: Option<SocketAddr>, _headers: &HeaderMap) -> CookieScope {
        CookieScope::default()
    }
}

#[derive(Default)]
struct Recorder {
    events: Mutex<Vec<PortalEvent>>,
}

#[async_trait]
impl EventSink for Recorder {
    fn emit(&self, event: PortalEvent) {
        self.events.lock().unwrap().push(event);
    }

    async fn settle(&self, _within: Duration) {}
}

impl Recorder {
    fn named(&self, name: EventName) -> Vec<PortalEvent> {
        self.events
            .lock()
            .unwrap()
            .iter()
            .filter(|event| event.name == name)
            .cloned()
            .collect()
    }
}

struct Portal {
    router: Router,
    events: Arc<Recorder>,
    _directory: TempDir,
}

async fn require(gate: Arc<dyn Gate>, request: Request<Body>, next: Next) -> Response {
    match gate.admit(request.headers()) {
        Ok(_) => next.run(request).await,
        Err(error) => axum::response::IntoResponse::into_response(error),
    }
}

fn portal() -> Portal {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    let hash = hash_password("secret").unwrap();
    fs::write(
        &path,
        format!("[[users]]\nname = \"admin\"\npassword_hash = \"{hash}\"\n"),
    )
    .unwrap();
    let store = Arc::new(ConfigStore::open(&path).unwrap());
    let events = Arc::new(Recorder::default());
    let feature = AuthFeature::new(store, Arc::new(Direct), events.clone());
    let gate = feature.gate();
    let protected = feature
        .router()
        .route_layer(middleware::from_fn(move |request, next| {
            require(gate.clone(), request, next)
        }));
    let home = Environment::parse("home").unwrap();
    let detected = DetectedEnvironment::new(home, &Environments::default());
    Portal {
        router: protected
            .merge(feature.public_router())
            .layer(axum::Extension(detected))
            .layer(axum::Extension(Environment::parse("vpn").unwrap())),
        events,
        _directory: directory,
    }
}

fn sign_in_request(name: &str, password: &str) -> Request<Body> {
    Request::post(AuthFeature::PATH)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(format!(
            r#"{{"name":"{name}","password":"{password}"}}"#
        )))
        .unwrap()
}

fn with_cookie(method: &str, cookie: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(AuthFeature::PATH)
        .header(COOKIE, cookie)
        .body(Body::empty())
        .unwrap()
}

async fn body_of(response: Response) -> String {
    String::from_utf8(
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap()
}

#[tokio::test]
async fn sign_in_who_am_i_sign_out_and_then_the_session_is_gone() {
    let portal = portal();
    let signed_in = portal
        .router
        .clone()
        .oneshot(sign_in_request("admin", "secret"))
        .await
        .unwrap();
    assert_eq!(signed_in.status(), StatusCode::OK);
    let cookie = signed_in.headers()[SET_COOKIE]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string();

    let who = portal
        .router
        .clone()
        .oneshot(with_cookie("GET", &cookie))
        .await
        .unwrap();
    assert_eq!(who.status(), StatusCode::OK);
    assert_eq!(body_of(who).await, r#"{"name":"admin"}"#);

    let out = portal
        .router
        .clone()
        .oneshot(with_cookie("DELETE", &cookie))
        .await
        .unwrap();
    assert_eq!(out.status(), StatusCode::NO_CONTENT);
    assert!(
        out.headers()[SET_COOKIE]
            .to_str()
            .unwrap()
            .contains("Max-Age=0")
    );

    let after = portal
        .router
        .clone()
        .oneshot(with_cookie("GET", &cookie))
        .await
        .unwrap();
    assert_eq!(after.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn the_session_cookie_is_http_only_and_same_site_strict() {
    let response = portal()
        .router
        .oneshot(sign_in_request("admin", "secret"))
        .await
        .unwrap();
    let cookie = response.headers()[SET_COOKIE].to_str().unwrap();
    assert!(cookie.starts_with("home_portal_session="));
    assert!(cookie.contains("HttpOnly"));
    assert!(cookie.contains("SameSite=Strict"));
    assert!(cookie.contains("Path=/"));
}

#[tokio::test]
async fn an_unknown_name_and_a_wrong_password_answer_the_same() {
    let portal = portal();
    let unknown = portal
        .router
        .clone()
        .oneshot(sign_in_request("nobody", "secret"))
        .await
        .unwrap();
    let wrong = portal
        .router
        .clone()
        .oneshot(sign_in_request("admin", "wrong"))
        .await
        .unwrap();
    assert_eq!(unknown.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(wrong.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(body_of(unknown).await, body_of(wrong).await);
}

#[tokio::test]
async fn the_sixth_attempt_is_throttled_even_with_the_right_password() {
    let portal = portal();
    for _ in 0..5 {
        let failed = portal
            .router
            .clone()
            .oneshot(sign_in_request("admin", "wrong"))
            .await
            .unwrap();
        assert_eq!(failed.status(), StatusCode::UNAUTHORIZED);
    }
    let throttled = portal
        .router
        .clone()
        .oneshot(sign_in_request("admin", "secret"))
        .await
        .unwrap();
    assert_eq!(throttled.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(throttled.headers().contains_key(RETRY_AFTER));
}

#[tokio::test]
async fn signing_out_requires_a_session() {
    let response = portal()
        .router
        .oneshot(with_cookie("DELETE", "home_portal_session=forged"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_sign_in_and_a_sign_out_are_announced_from_the_detected_environment() {
    let portal = portal();
    let signed_in = portal
        .router
        .clone()
        .oneshot(sign_in_request("admin", "secret"))
        .await
        .unwrap();
    let cookie = signed_in.headers()[SET_COOKIE]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string();
    portal
        .router
        .clone()
        .oneshot(with_cookie("DELETE", &cookie))
        .await
        .unwrap();
    let signed_in = portal.events.named(EventName::UserSignedIn);
    assert_eq!(signed_in.len(), 1);
    assert_eq!(signed_in[0].value("user.name"), Some("admin"));
    assert_eq!(signed_in[0].value("client.address"), Some("127.0.0.1"));
    assert_eq!(signed_in[0].value("client.environment"), Some("home"));
    let signed_out = portal.events.named(EventName::UserSignedOut);
    assert_eq!(signed_out.len(), 1);
    assert_eq!(signed_out[0].value("user.name"), Some("admin"));
}

#[tokio::test]
async fn an_unknown_name_and_a_wrong_password_are_announced_alike() {
    let portal = portal();
    for (name, password) in [("nobody", "secret"), ("admin", "wrong")] {
        portal
            .router
            .clone()
            .oneshot(sign_in_request(name, password))
            .await
            .unwrap();
    }
    let failed = portal.events.named(EventName::UserSignInFailed);
    assert_eq!(failed.len(), 2);
    for event in &failed {
        assert_eq!(event.value("sign_in.reason"), Some("credentials"));
    }
    assert_eq!(failed[0].value("user.name"), Some("nobody"));
    assert!(portal.events.named(EventName::UserSignedIn).is_empty());
}

#[tokio::test]
async fn a_throttled_attempt_is_announced_as_throttled() {
    let portal = portal();
    for _ in 0..6 {
        portal
            .router
            .clone()
            .oneshot(sign_in_request("admin", "wrong"))
            .await
            .unwrap();
    }
    let failed = portal.events.named(EventName::UserSignInFailed);
    assert_eq!(failed.len(), 6);
    assert_eq!(failed[5].value("sign_in.reason"), Some("throttled"));
    assert_eq!(failed[4].value("sign_in.reason"), Some("credentials"));
}
