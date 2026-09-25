use std::fs;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::header::{CONTENT_TYPE, ETAG, IF_MATCH};
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use portal_config::ConfigStore;
use portal_feature::Feature;
use serde_json::Value;
use tempfile::TempDir;
use tower::ServiceExt;

use super::NetworkFeature;
use crate::types::EffectiveAddress;

struct Portal {
    router: Router,
    path: std::path::PathBuf,
    _directory: TempDir,
}

fn portal(effective: &str, overridden: bool) -> Portal {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    fs::write(
        &path,
        "# portal\n[network]\n# keep this local\naddress = \"127.0.0.1\"\nport = 8080\n",
    )
    .unwrap();
    let store = Arc::new(ConfigStore::open(&path).unwrap());
    let feature = NetworkFeature::new(
        store.clone(),
        EffectiveAddress {
            address: effective.parse().unwrap(),
            overridden,
        },
    );
    store.adopt(vec![feature.validator().unwrap()]).unwrap();
    Portal {
        router: feature.router(),
        path,
        _directory: directory,
    }
}

async fn get(router: &Router) -> (String, Value) {
    let response = router
        .clone()
        .oneshot(
            Request::get(NetworkFeature::PATH)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let etag = response.headers()[ETAG].to_str().unwrap().to_string();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (etag, serde_json::from_slice(&bytes).unwrap())
}

async fn put(router: &Router, revision: &str, body: &str) -> (StatusCode, Value) {
    let request = Request::put(NetworkFeature::PATH)
        .header(CONTENT_TYPE, "application/json")
        .header(IF_MATCH, revision)
        .body(Body::from(body.to_string()))
        .unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[tokio::test]
async fn the_settings_show_configured_effective_and_interfaces() {
    let (_, body) = get(&portal("127.0.0.1:8080", false).router).await;
    assert_eq!(body["configured"]["port"], 8080);
    assert_eq!(body["effective"]["address"], "127.0.0.1:8080");
    assert_eq!(body["restart_required"], false);
    assert!(!body["interfaces"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn changing_the_port_writes_the_file_and_asks_for_a_restart() {
    let portal = portal("127.0.0.1:8080", false);
    let (revision, _) = get(&portal.router).await;
    let (status, body) = put(
        &portal.router,
        &revision,
        r#"{"address":"127.0.0.1","port":9090,"trusted_proxies":[]}"#,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["restart_required"], true);
    assert_eq!(body["effective"]["address"], "127.0.0.1:8080");
    let text = fs::read_to_string(&portal.path).unwrap();
    assert!(text.contains("port = 9090"));
    assert!(text.contains("# keep this local"));
}

#[tokio::test]
async fn an_environment_override_is_reported() {
    let (_, body) = get(&portal("0.0.0.0:9000", true).router).await;
    assert_eq!(body["effective"]["address"], "0.0.0.0:9000");
    assert_eq!(body["effective"]["overridden"], true);
    assert_eq!(body["restart_required"], false);
}

#[tokio::test]
async fn a_bad_range_of_trusted_proxies_is_refused_by_field() {
    let portal = portal("127.0.0.1:8080", false);
    let (revision, _) = get(&portal.router).await;
    let (status, body) = put(
        &portal.router,
        &revision,
        r#"{"address":"127.0.0.1","port":8080,"trusted_proxies":["10.0.0.0/33"]}"#,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["errors"][0]["field"], "trusted_proxies[0]");
}
