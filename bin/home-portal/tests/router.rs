mod support;

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::header::{CONTENT_TYPE, COOKIE, SET_COOKIE};
use axum::http::{HeaderMap, Request, StatusCode};
use axum::routing::{get, post};
use home_portal::{Registry, assemble, registered};
use portal_feature::{ApiError, Feature, Gate, Principal};
use tower::ServiceExt;

const PROBE_PATH: &str = "/api/probe";
const OPEN_PATH: &str = "/api/open";
const TICKET: &str = "ticket=yes";

struct ProbeFeature;

impl Feature for ProbeFeature {
    fn name(&self) -> &'static str {
        "probe"
    }

    fn router(&self) -> Router {
        Router::new().route(
            PROBE_PATH,
            get(|| async { "probe" }).post(|| async { "posted" }),
        )
    }

    fn public_router(&self) -> Router {
        Router::new().route(
            OPEN_PATH,
            post(|| async { "open" }).get(|| async { "open" }),
        )
    }
}

struct Ticket;

impl Gate for Ticket {
    fn admit(&self, headers: &HeaderMap) -> Result<Principal, ApiError> {
        match headers.get(COOKIE).and_then(|value| value.to_str().ok()) {
            Some(TICKET) => Ok(Principal {
                name: "tester".into(),
            }),
            _ => Err(ApiError::Unauthorized),
        }
    }
}

fn probe_configuration() -> Arc<portal_config::ConfigStore> {
    let directory = Box::leak(Box::new(tempfile::tempdir().unwrap()));
    let path = support::with_extra(directory, "secret", "");
    Arc::new(portal_config::ConfigStore::open(path).unwrap())
}

fn probe_portal() -> Router {
    let configuration = probe_configuration();
    assemble(&Registry {
        features: vec![
            Arc::new(ProbeFeature),
            Arc::new(portal_health::HealthFeature),
        ],
        gate: Arc::new(Ticket),
        configuration: configuration.clone(),
        widgets: Arc::new(portal_widget::WidgetRegistry::new(
            configuration,
            Vec::new(),
        )),
        events: Arc::new(Silent),
    })
}

async fn status_of(router: Router, request: Request<Body>) -> StatusCode {
    router.oneshot(request).await.unwrap().status()
}

fn get_request(path: &str) -> Request<Body> {
    Request::get(path).body(Body::empty()).unwrap()
}

#[tokio::test]
async fn a_protected_route_answers_401_without_a_session_and_200_with_one() {
    assert_eq!(
        status_of(probe_portal(), get_request(PROBE_PATH)).await,
        StatusCode::UNAUTHORIZED
    );
    let with_ticket = Request::get(PROBE_PATH)
        .header(COOKIE, TICKET)
        .body(Body::empty())
        .unwrap();
    assert_eq!(status_of(probe_portal(), with_ticket).await, StatusCode::OK);
}

#[tokio::test]
async fn a_public_route_and_the_liveness_endpoint_need_no_session() {
    assert_eq!(
        status_of(probe_portal(), get_request(OPEN_PATH)).await,
        StatusCode::OK
    );
    assert_eq!(
        status_of(probe_portal(), get_request("/health")).await,
        StatusCode::OK
    );
}

#[tokio::test]
async fn a_form_post_to_the_api_is_refused_with_415() {
    let form = Request::post(OPEN_PATH)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header("content-length", "3")
        .body(Body::from("a=b"))
        .unwrap();
    assert_eq!(
        status_of(probe_portal(), form).await,
        StatusCode::UNSUPPORTED_MEDIA_TYPE
    );
    let json = Request::post(OPEN_PATH)
        .header(CONTENT_TYPE, "application/json")
        .header("content-length", "2")
        .body(Body::from("{}"))
        .unwrap();
    assert_eq!(status_of(probe_portal(), json).await, StatusCode::OK);
}

#[tokio::test]
async fn an_api_path_no_feature_serves_is_not_found() {
    assert_eq!(
        status_of(probe_portal(), get_request("/api/does-not-exist")).await,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        status_of(probe_portal(), get_request("/api")).await,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn a_path_outside_the_api_is_answered_by_the_interface() {
    let response = probe_portal()
        .oneshot(get_request("/services/"))
        .await
        .unwrap();
    let status = response.status();
    assert!(
        status == StatusCode::OK || status == StatusCode::SERVICE_UNAVAILABLE,
        "the interface fallback answers 200 when built and 503 when not, got {status}"
    );
    assert_ne!(
        response
            .headers()
            .get(CONTENT_TYPE)
            .map(|value| value.to_str().unwrap()),
        Some("application/json")
    );
}

#[test]
#[should_panic(expected = "Overlapping method route")]
fn two_features_claiming_one_path_fail_assembly() {
    let configuration = probe_configuration();
    let _conflicting = assemble(&Registry {
        features: vec![Arc::new(ProbeFeature), Arc::new(ProbeFeature)],
        gate: Arc::new(Ticket),
        configuration: configuration.clone(),
        widgets: Arc::new(portal_widget::WidgetRegistry::new(
            configuration,
            Vec::new(),
        )),
        events: Arc::new(Silent),
    });
}

#[tokio::test]
async fn the_real_portal_protects_the_api_and_signs_in_through_the_session_route() {
    let directory = tempfile::tempdir().unwrap();
    let path = support::with_extra(&directory, "secret", "");
    let registry = registered(&support::wiring_for(&path)).unwrap();
    let portal = assemble(&registry);
    assert_eq!(
        status_of(portal.clone(), get_request("/api/session")).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        status_of(portal.clone(), get_request("/api/services")).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        status_of(
            portal.clone(),
            Request::post("/api/services/router/probe")
                .body(Body::empty())
                .unwrap()
        )
        .await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        status_of(
            portal.clone(),
            Request::post("/api/icon-preview")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"icon":"catalog:jellyfin","url":null}"#))
                .unwrap()
        )
        .await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        status_of(
            portal.clone(),
            Request::put("/api/proxy/caddy")
                .header("content-type", "application/json")
                .header("if-match", "\"r\"")
                .body(Body::from(r#"{"version":"2.10.2"}"#))
                .unwrap()
        )
        .await,
        StatusCode::UNAUTHORIZED
    );
    let sign_in = Request::post("/api/session")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"name":"admin","password":"secret"}"#))
        .unwrap();
    let response = portal.clone().oneshot(sign_in).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let cookie = response.headers()[SET_COOKIE]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string();
    let listed = Request::get("/api/services")
        .header(COOKIE, &cookie)
        .body(Body::empty())
        .unwrap();
    let response = portal.clone().oneshot(listed).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = http_body_util::BodyExt::collect(response.into_body())
        .await
        .unwrap()
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["services"][0]["id"], "router");
    assert_eq!(json["services"][0]["status"]["state"], "unknown");
}

struct Silent;

#[async_trait::async_trait]
impl portal_feature::EventSink for Silent {
    fn emit(&self, _event: portal_feature::PortalEvent) {}

    async fn settle(&self, _within: std::time::Duration) {}
}

#[tokio::test]
async fn following_and_stopping_a_run_need_a_session() {
    let directory = tempfile::tempdir().unwrap();
    let path = support::with_extra(&directory, "secret", "");
    let registry = registered(&support::wiring_for(&path)).unwrap();
    let portal = assemble(&registry);
    assert_eq!(
        status_of(portal.clone(), get_request("/api/automations/runs/1")).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        status_of(
            portal,
            Request::post("/api/automations/runs/1/stop")
                .body(Body::empty())
                .unwrap()
        )
        .await,
        StatusCode::UNAUTHORIZED
    );
}
