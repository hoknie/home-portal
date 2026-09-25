use std::fs;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::header::RETRY_AFTER;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use portal_config::ConfigStore;
use portal_feature::Feature;
use portal_model::Environment;
use tempfile::TempDir;
use tower::ServiceExt;

use crate::ServicesFeature;
use crate::fakes::{Behaviour, Upstream};

struct Portal {
    router: Router,
    _directory: TempDir,
}

fn portal_with(text: &str) -> Portal {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    fs::write(&path, text).unwrap();
    let store = Arc::new(ConfigStore::open(&path).unwrap());
    let feature = ServicesFeature::new(
        store.clone(),
        Environment::internet(),
        Vec::new(),
        Arc::new(crate::fakes::Switch::on()),
    )
    .unwrap();
    store.adopt(vec![feature.validator().unwrap()]).unwrap();
    Portal {
        router: feature.router(),
        _directory: directory,
    }
}

async fn post(portal: &Portal, uri: &str, environment: Environment) -> Response {
    let mut request = Request::post(uri).body(Body::empty()).unwrap();
    request.extensions_mut().insert(environment);
    portal.router.clone().oneshot(request).await.unwrap()
}

fn services(url: &str) -> String {
    format!(
        "[environments.local]\nnetworks = [\"192.168.0.0/16\"]\n\n[[services]]\nid = \"media\"\nname = \"Media\"\nurl = \"{url}\"\nprobe = {{ every_seconds = 60, timeout_seconds = 1 }}\n\n[[services]]\nid = \"printer\"\nname = \"Printer\"\nurl = \"http://10.255.0.3\"\nprobe = {{ enabled = false }}\n\n[[services]]\nid = \"nas\"\nname = \"NAS\"\nurl = \"http://10.255.0.4\"\nenvironments = [\"local\"]\nprobe = {{ enabled = false }}\n"
    )
}

#[tokio::test]
async fn asking_for_a_probe_answers_202_and_the_probe_runs_at_once() {
    let upstream = Upstream::start(Behaviour::Status {
        code: 200,
        delay: std::time::Duration::ZERO,
    })
    .await;
    let portal = portal_with(&services(&upstream.url()));
    let response = post(
        &portal,
        "/api/services/media/probe",
        Environment::internet(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::ACCEPTED);
    for _ in 0..50 {
        if upstream.hits() >= 1 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert!(upstream.hits() >= 1);
}

#[tokio::test]
async fn a_second_request_within_five_seconds_is_throttled_with_retry_after() {
    let portal = portal_with(&services("http://127.0.0.1:1"));
    assert_eq!(
        post(
            &portal,
            "/api/services/media/probe",
            Environment::internet()
        )
        .await
        .status(),
        StatusCode::ACCEPTED
    );
    let second = post(
        &portal,
        "/api/services/media/probe",
        Environment::internet(),
    )
    .await;
    assert_eq!(second.status(), StatusCode::TOO_MANY_REQUESTS);
    let retry: u64 = second.headers()[RETRY_AFTER]
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!((1..=5).contains(&retry));
}

#[tokio::test]
async fn an_unknown_or_hidden_service_is_404_and_a_disabled_one_is_409() {
    let portal = portal_with(&services("http://127.0.0.1:1"));
    let cases = [
        ("/api/services/missing/probe", StatusCode::NOT_FOUND),
        ("/api/services/nas/probe", StatusCode::NOT_FOUND),
        ("/api/services/printer/probe", StatusCode::CONFLICT),
    ];
    for (uri, status) in cases {
        assert_eq!(
            post(&portal, uri, Environment::internet()).await.status(),
            status,
            "{uri}"
        );
    }
}

async fn get(
    portal: &Portal,
    uri: &str,
    environment: Environment,
) -> (StatusCode, serde_json::Value) {
    let mut request = Request::get(uri).body(Body::empty()).unwrap();
    request.extensions_mut().insert(environment);
    let response = portal.router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = http_body_util::BodyExt::collect(response.into_body())
        .await
        .unwrap()
        .to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null),
    )
}

#[tokio::test]
async fn the_history_answers_for_each_range_with_uptime_for_all_three() {
    let portal = portal_with(&services("http://127.0.0.1:1"));
    for range in ["24h", "7d", "30d"] {
        let (status, body) = get(
            &portal,
            &format!("/api/services/media/history?range={range}"),
            Environment::internet(),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{range}");
        assert_eq!(body["range"], range);
        assert_eq!(body["uptime"].as_array().unwrap().len(), 3);
        assert!(body["points"].is_array());
        assert!(body["transitions"].is_array());
    }
    let (status, body) = get(
        &portal,
        "/api/services/media/history",
        Environment::internet(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["range"], "24h");
}

#[tokio::test]
async fn an_unknown_range_is_400_and_an_unknown_or_hidden_service_is_404() {
    let portal = portal_with(&services("http://127.0.0.1:1"));
    let cases = [
        (
            "/api/services/media/history?range=1y",
            StatusCode::BAD_REQUEST,
        ),
        ("/api/services/missing/history", StatusCode::NOT_FOUND),
        ("/api/services/nas/history", StatusCode::NOT_FOUND),
    ];
    for (uri, status) in cases {
        assert_eq!(
            get(&portal, uri, Environment::internet()).await.0,
            status,
            "{uri}"
        );
    }
    assert_eq!(
        get(
            &portal,
            "/api/services/nas/history",
            Environment::parse("local").unwrap()
        )
        .await
        .0,
        StatusCode::OK
    );
}

const PUBLISHED: &str = "[environments.local]\nnetworks = [\"192.168.0.0/16\"]\n\n[environments.vpn]\nnetworks = [\"10.8.0.0/24\"]\n\n[[services]]\nid = \"media\"\nname = \"Media\"\nurl = \"http://192.168.1.10:8096\"\nproxy = { host = \"media.example.com\", environments = [\"internet\", \"vpn\"] }\nprobe = { enabled = false }\n";

fn published_portal(switch: Arc<crate::fakes::Switch>) -> (Router, std::path::PathBuf, TempDir) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    fs::write(&path, PUBLISHED).unwrap();
    let store = Arc::new(ConfigStore::open(&path).unwrap());
    let feature =
        ServicesFeature::new(store.clone(), Environment::internet(), Vec::new(), switch).unwrap();
    store.adopt(vec![feature.validator().unwrap()]).unwrap();
    (feature.router(), path, directory)
}

async fn shown_address(router: &Router, environment: &str) -> String {
    let mut request = Request::get(ServicesFeature::COLLECTION)
        .body(Body::empty())
        .unwrap();
    request
        .extensions_mut()
        .insert(Environment::parse(environment).unwrap());
    let response = router.clone().oneshot(request).await.unwrap();
    let bytes = http_body_util::BodyExt::collect(response.into_body())
        .await
        .unwrap()
        .to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    body["services"][0]["address"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn a_published_environment_is_shown_the_address_of_the_proxy() {
    let (router, _, _directory) = published_portal(Arc::new(crate::fakes::Switch::on()));
    assert_eq!(
        shown_address(&router, "internet").await,
        "https://media.example.com"
    );
    assert_eq!(
        shown_address(&router, "vpn").await,
        "https://media.example.com"
    );
    assert_eq!(
        shown_address(&router, "local").await,
        "http://192.168.1.10:8096"
    );
}

#[tokio::test]
async fn with_publishing_switched_off_every_visitor_is_shown_the_url() {
    let (router, _, _directory) = published_portal(Arc::new(crate::fakes::Switch::default()));
    assert_eq!(
        shown_address(&router, "internet").await,
        "http://192.168.1.10:8096"
    );
    assert_eq!(
        shown_address(&router, "vpn").await,
        "http://192.168.1.10:8096"
    );
}

#[tokio::test]
async fn a_publication_saved_through_the_api_is_reported_with_its_defaults_and_written() {
    let (router, path, _directory) = published_portal(Arc::new(crate::fakes::Switch::on()));
    let listing = router
        .clone()
        .oneshot({
            let mut request = Request::get(ServicesFeature::COLLECTION)
                .body(Body::empty())
                .unwrap();
            request.extensions_mut().insert(Environment::internet());
            request
        })
        .await
        .unwrap();
    let revision = listing.headers()["etag"].to_str().unwrap().to_string();
    let mut request = Request::put("/api/services/media")
        .header("content-type", "application/json")
        .header("if-match", revision)
        .body(Body::from(
            r#"{"id":"media","name":"Media","url":"http://192.168.1.10:8096","probe":{"enabled":false},"proxy":{"host":" Media.Example.com ","auth":["internet"]}}"#,
        ))
        .unwrap();
    request.extensions_mut().insert(Environment::internet());
    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = http_body_util::BodyExt::collect(response.into_body())
        .await
        .unwrap()
        .to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        body["proxy"],
        serde_json::json!({"host":"media.example.com","upstream":null,"environments":["internet"],"auth":["internet"],"tls":null,"upstream_verify":true})
    );
    assert!(
        fs::read_to_string(&path)
            .unwrap()
            .contains("proxy = { host = \"media.example.com\", auth = [\"internet\"] }\n")
    );
}

#[tokio::test]
async fn a_host_another_service_publishes_is_refused_on_the_host_field() {
    let text = format!(
        "{PUBLISHED}\n[[services]]\nid = \"nas\"\nname = \"NAS\"\nurl = \"http://192.168.1.5\"\nprobe = {{ enabled = false }}\n"
    );
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    fs::write(&path, text).unwrap();
    let store = Arc::new(ConfigStore::open(&path).unwrap());
    let feature = ServicesFeature::new(
        store.clone(),
        Environment::internet(),
        Vec::new(),
        Arc::new(crate::fakes::Switch::on()),
    )
    .unwrap();
    let revision = store.read().revision.as_str().to_string();
    let mut request = Request::put("/api/services/nas")
        .header("content-type", "application/json")
        .header("if-match", format!("\"{revision}\""))
        .body(Body::from(
            r#"{"id":"nas","name":"NAS","url":"http://192.168.1.5","probe":{"enabled":false},"proxy":{"host":"media.example.com"}}"#,
        ))
        .unwrap();
    request.extensions_mut().insert(Environment::internet());
    let response = feature.router().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let bytes = http_body_util::BodyExt::collect(response.into_body())
        .await
        .unwrap()
        .to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["errors"][0]["field"], "proxy.host");
}
