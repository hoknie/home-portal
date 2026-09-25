mod support;

use std::net::SocketAddr;

use axum::Router;
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::header::{CONTENT_TYPE, COOKIE, SET_COOKIE};
use axum::http::{Request, StatusCode};
use home_portal::{assemble, registered};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

const ENVIRONMENTS: &str = r#"
[environments.local]
networks = ["192.168.0.0/16"]

[environments.vpn]
networks = ["10.8.0.0/24"]

[[services]]
id = "nas"
name = "NAS"
url = "http://192.168.1.60"
addresses = { internet = "https://nas.example.com" }
environments = ["local"]
probe = { enabled = false }

[[services]]
id = "media"
name = "Media"
url = "http://192.168.1.10"
addresses = { internet = "https://media.example.com" }
probe = { enabled = false }
"#;

async fn signed_in_portal() -> (Router, String) {
    let directory = Box::leak(Box::new(tempfile::tempdir().unwrap()));
    let path = support::with_extra(directory, "secret", ENVIRONMENTS);
    let text = std::fs::read_to_string(&path).unwrap().replace(
        "trusted_proxies = []",
        r#"trusted_proxies = ["172.17.0.0/16"]"#,
    );
    std::fs::write(&path, text).unwrap();
    let router = assemble(&registered(&support::wiring_for(&path)).unwrap());
    let sign_in = Request::post("/api/session")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"name":"admin","password":"secret"}"#))
        .unwrap();
    let response = router.clone().oneshot(sign_in).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let cookie = response.headers()[SET_COOKIE]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string();
    (router, cookie)
}

async fn environment_of(
    router: &Router,
    cookie: &str,
    peer: &str,
    forwarded: Option<&str>,
) -> Value {
    fetch(router, "/api/environment", cookie, peer, forwarded).await
}

async fn fetch(
    router: &Router,
    path: &str,
    cookie: &str,
    peer: &str,
    forwarded: Option<&str>,
) -> Value {
    let mut builder = Request::get(path).header(COOKIE, cookie);
    if let Some(forwarded) = forwarded {
        builder = builder.header("x-forwarded-for", forwarded);
    }
    let mut request = builder.body(Body::empty()).unwrap();
    let address: SocketAddr = peer.parse().unwrap();
    request.extensions_mut().insert(ConnectInfo(address));
    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn a_visitor_is_placed_by_their_address() {
    let (router, cookie) = signed_in_portal().await;
    let home = environment_of(&router, &cookie, "192.168.1.40:5000", None).await;
    assert_eq!(home["environment"], "local");
    assert_eq!(
        home["environments"]
            .as_array()
            .unwrap()
            .iter()
            .map(|name| name.as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["local", "vpn", "internet"]
    );
    assert_eq!(
        environment_of(&router, &cookie, "10.8.0.9:5000", None).await["environment"],
        "vpn"
    );
    assert_eq!(
        environment_of(&router, &cookie, "203.0.113.5:5000", None).await["environment"],
        "internet"
    );
}

#[tokio::test]
async fn a_trusted_proxy_passes_on_the_visitor_address() {
    let (router, cookie) = signed_in_portal().await;
    let through_proxy =
        environment_of(&router, &cookie, "172.17.0.2:5000", Some("192.168.1.40")).await;
    assert_eq!(through_proxy["environment"], "local");
    let forged = environment_of(&router, &cookie, "203.0.113.5:5000", Some("192.168.1.40")).await;
    assert_eq!(forged["environment"], "internet");
}

fn service<'a>(services: &'a Value, id: &str) -> Option<&'a Value> {
    services["services"]
        .as_array()
        .unwrap()
        .iter()
        .find(|service| service["id"] == id)
}

#[tokio::test]
async fn a_visitor_without_a_choice_is_reported_in_both_environments() {
    let (router, cookie) = signed_in_portal().await;
    let home = environment_of(&router, &cookie, "192.168.1.40:5000", None).await;
    assert_eq!(home["environment"], "local");
    assert_eq!(home["detected"], "local");
    assert_eq!(home["switchable"], true);
}

#[tokio::test]
async fn a_visitor_at_home_may_look_as_from_outside() {
    let (router, cookie) = signed_in_portal().await;
    let chosen = format!("{cookie}; portal_environment=internet");
    let reported = environment_of(&router, &chosen, "192.168.1.40:5000", None).await;
    assert_eq!(reported["environment"], "internet");
    assert_eq!(reported["detected"], "local");
    assert_eq!(reported["switchable"], true);
    let services = fetch(&router, "/api/services", &chosen, "192.168.1.40:5000", None).await;
    assert!(service(&services, "nas").is_none());
    assert_eq!(
        service(&services, "media").unwrap()["address"],
        "https://media.example.com"
    );
}

#[tokio::test]
async fn a_choice_from_outside_is_ignored() {
    let (router, cookie) = signed_in_portal().await;
    let chosen = format!("{cookie}; portal_environment=local");
    let reported = environment_of(&router, &chosen, "203.0.113.5:5000", None).await;
    assert_eq!(reported["environment"], "internet");
    assert_eq!(reported["detected"], "internet");
    assert_eq!(reported["switchable"], false);
    let services = fetch(&router, "/api/services", &chosen, "203.0.113.5:5000", None).await;
    assert!(service(&services, "nas").is_none());
}

#[tokio::test]
async fn a_choice_naming_no_environment_is_ignored() {
    let (router, cookie) = signed_in_portal().await;
    for value in ["office", "NOT%20VALID", ""] {
        let chosen = format!("{cookie}; portal_environment={value}");
        let reported = environment_of(&router, &chosen, "192.168.1.40:5000", None).await;
        assert_eq!(reported["environment"], "local", "{value}");
    }
}

#[tokio::test]
async fn sign_in_throttling_counts_the_address_whatever_the_choice() {
    let (router, _) = signed_in_portal().await;
    let attempt = |cookie: Option<&str>| {
        let mut builder = Request::post("/api/session").header(CONTENT_TYPE, "application/json");
        if let Some(cookie) = cookie {
            builder = builder.header(COOKIE, cookie);
        }
        let mut request = builder
            .body(Body::from(r#"{"name":"admin","password":"wrong"}"#))
            .unwrap();
        let address: SocketAddr = "192.168.1.40:5000".parse().unwrap();
        request.extensions_mut().insert(ConnectInfo(address));
        router.clone().oneshot(request)
    };
    for _ in 0..5 {
        let response = attempt(Some("portal_environment=internet")).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
    let locked = attempt(None).await.unwrap();
    assert_eq!(locked.status(), StatusCode::TOO_MANY_REQUESTS);
}
