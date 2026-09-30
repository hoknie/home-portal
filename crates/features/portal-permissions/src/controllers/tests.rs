use std::fs;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use portal_config::ConfigStore;
use portal_feature::Feature;
use portal_model::{DetectedEnvironment, Environment, Environments};
use serde_json::Value;
use tempfile::TempDir;
use tower::ServiceExt;

use super::permissions::OUTSIDE;
use crate::fakes::FixedChecks;
use crate::features::PermissionsFeature;
use crate::types::{Limits, Owner};

struct Api {
    router: Router,
    _folder: TempDir,
}

fn api(delay: Duration) -> Api {
    let folder = TempDir::new().unwrap();
    let path = folder.path().join("home-portal.toml");
    fs::write(&path, "[permissions]\nautomation = [\"Finder\"]\n").unwrap();
    let store = Arc::new(ConfigStore::open(&path).unwrap());
    let feature = PermissionsFeature::with(
        store,
        Arc::new(FixedChecks::granting(delay)),
        Owner::detect(Some("Apple_Terminal".to_string()), None),
        Limits::default(),
    );
    Api {
        router: feature.router(),
        _folder: folder,
    }
}

fn from(environment: &str) -> DetectedEnvironment {
    DetectedEnvironment::new(
        Environment::parse(environment).unwrap(),
        &Environments::default(),
    )
}

async fn send(api: &Api, method: &str, uri: &str, environment: &str) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    request.extensions_mut().insert(from(environment));
    let response = api.router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = serde_json::from_slice(&bytes)
        .unwrap_or(Value::String(String::from_utf8_lossy(&bytes).to_string()));
    (status, body)
}

#[tokio::test]
async fn the_states_are_listed_with_their_pane_and_owner() {
    let api = api(Duration::ZERO);
    let (status, body) = send(&api, "GET", PermissionsFeature::PATH, "home").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["owner"]["kind"], "terminal");
    assert_eq!(body["owner"]["name"], "Terminal");
    let codes: Vec<&str> = body["permissions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|permission| permission["code"].as_str().unwrap())
        .collect();
    assert!(codes.contains(&"automation:Finder"), "{codes:?}");
    assert_eq!(body["permissions"][0]["state"], "unknown");
    assert_eq!(body["permissions"][0]["pane"], "local-network");
    assert_eq!(body["permissions"][0]["learned_at"], Value::Null);
}

#[tokio::test]
async fn a_request_from_inside_is_accepted_and_its_answers_appear_later() {
    let api = api(Duration::ZERO);
    let (status, _) = send(&api, "POST", PermissionsFeature::REQUEST, "home").await;
    assert_eq!(status, StatusCode::ACCEPTED);
    for _ in 0..100 {
        let (_, body) = send(&api, "GET", PermissionsFeature::PATH, "home").await;
        if body["permissions"][0]["state"] == "granted" {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("the request never answered");
}

#[tokio::test]
async fn a_second_request_while_one_runs_is_a_conflict() {
    let api = api(Duration::from_millis(300));
    let (first, _) = send(&api, "POST", PermissionsFeature::REQUEST, "home").await;
    let (second, _) = send(&api, "POST", PermissionsFeature::REQUEST, "home").await;
    assert_eq!(first, StatusCode::ACCEPTED);
    assert_eq!(second, StatusCode::CONFLICT);
    let (_, listed) = send(&api, "GET", PermissionsFeature::PATH, "home").await;
    assert_eq!(listed["permissions"][0]["state"], "pending");
}

#[tokio::test]
async fn a_request_from_the_internet_is_forbidden_and_asks_nothing() {
    let api = api(Duration::ZERO);
    let (status, body) = send(&api, "POST", PermissionsFeature::REQUEST, "internet").await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body, Value::String(OUTSIDE.to_string()));
    let (_, listed) = send(&api, "GET", PermissionsFeature::PATH, "home").await;
    assert_eq!(listed["permissions"][0]["state"], "unknown");
}
