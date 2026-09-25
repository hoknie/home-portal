use std::fs;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::http::header::{CONTENT_TYPE, ETAG, IF_MATCH};
use axum::http::{Request, StatusCode};
use axum::response::Response;
use http_body_util::BodyExt;
use portal_config::ConfigStore;
use portal_feature::Feature;
use portal_model::Environment;
use serde_json::Value;
use tempfile::TempDir;
use tower::ServiceExt;

use super::ServicesFeature;
use crate::fakes::{Behaviour, Upstream};

const FILE: &str = "# my services\n\n[[services]]\nid = \"b-first\"\nname = \"B\"\nurl = \"http://10.255.0.2\"\nprobe = { enabled = false }\n\n[[services]]\nid = \"a-second\"\nname = \"A\"\nurl = \"http://10.255.0.1\"\nprobe = { enabled = false }\n";

struct Portal {
    router: Router,
    path: std::path::PathBuf,
    _directory: TempDir,
}

fn portal() -> Portal {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    fs::write(&path, FILE).unwrap();
    let store = Arc::new(ConfigStore::open(&path).unwrap());
    let feature = ServicesFeature::new(
        store.clone(),
        portal_model::Environment::internet(),
        Vec::new(),
        Arc::new(crate::fakes::Switch::on()),
    )
    .unwrap();
    store.adopt(vec![feature.validator().unwrap()]).unwrap();
    Portal {
        router: feature.router(),
        path,
        _directory: directory,
    }
}

async fn send(router: &Router, request: Request<Body>) -> (StatusCode, Option<String>, Value) {
    send_from(router, request, Environment::internet()).await
}

async fn send_from(
    router: &Router,
    mut request: Request<Body>,
    environment: Environment,
) -> (StatusCode, Option<String>, Value) {
    request.extensions_mut().insert(environment);
    let response: Response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let etag = response
        .headers()
        .get(ETAG)
        .map(|value| value.to_str().unwrap().to_string());
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = serde_json::from_slice(&bytes)
        .unwrap_or(Value::String(String::from_utf8_lossy(&bytes).to_string()));
    (status, etag, body)
}

async fn revision(router: &Router) -> String {
    send(
        router,
        Request::get(ServicesFeature::COLLECTION)
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .1
    .unwrap()
}

fn write(method: &str, uri: &str, revision: Option<&str>, body: &str) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(CONTENT_TYPE, "application/json");
    if let Some(revision) = revision {
        builder = builder.header(IF_MATCH, revision);
    }
    builder.body(Body::from(body.to_string())).unwrap()
}

fn service_json(id: &str, url: &str) -> String {
    format!(
        r#"{{"id":"{id}","name":"New","url":"{url}","probe":{{"every_seconds":30,"timeout_seconds":1}}}}"#
    )
}

#[tokio::test]
async fn the_list_follows_the_file_order_and_carries_status_and_a_revision() {
    let portal = portal();
    let (status, etag, body) = send(
        &portal.router,
        Request::get(ServicesFeature::COLLECTION)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(etag.is_some());
    let ids: Vec<&str> = body["services"]
        .as_array()
        .unwrap()
        .iter()
        .map(|service| service["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, vec!["b-first", "a-second"]);
    assert_eq!(body["services"][0]["status"]["state"], "unknown");
    assert_eq!(body["services"][0]["probe"]["every_seconds"], 30);
}

#[tokio::test]
async fn adding_a_service_writes_the_file_keeps_comments_and_probes_it_within_a_second() {
    let portal = portal();
    let upstream = Upstream::start(Behaviour::Status {
        code: 200,
        delay: Duration::ZERO,
    })
    .await;
    let current = revision(&portal.router).await;
    let (status, etag, body) = send(
        &portal.router,
        write(
            "POST",
            ServicesFeature::COLLECTION,
            Some(&current),
            &service_json("added", &upstream.url()),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_ne!(etag.as_deref(), Some(current.as_str()));
    let text = fs::read_to_string(&portal.path).unwrap();
    assert!(text.starts_with("# my services\n"));
    assert!(text.contains("id = \"added\""));
    tokio::time::sleep(Duration::from_millis(1000)).await;
    assert!(upstream.hits() >= 1);
    let (_, _, listed) = send(
        &portal.router,
        Request::get(ServicesFeature::COLLECTION)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(listed["services"][2]["status"]["state"], "up");
}

#[tokio::test]
async fn a_write_without_a_revision_is_refused_with_428() {
    let portal = portal();
    let (status, _, _) = send(
        &portal.router,
        write(
            "POST",
            ServicesFeature::COLLECTION,
            None,
            &service_json("x", "http://x"),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::PRECONDITION_REQUIRED);
}

#[tokio::test]
async fn the_second_of_two_editors_gets_409_and_only_the_first_change_is_written() {
    let portal = portal();
    let read = revision(&portal.router).await;
    let first = send(
        &portal.router,
        write(
            "POST",
            ServicesFeature::COLLECTION,
            Some(&read),
            &service_json("first", "http://first"),
        ),
    )
    .await;
    assert_eq!(first.0, StatusCode::CREATED);
    let second = send(
        &portal.router,
        write(
            "POST",
            ServicesFeature::COLLECTION,
            Some(&read),
            &service_json("second", "http://second"),
        ),
    )
    .await;
    assert_eq!(second.0, StatusCode::CONFLICT);
    let text = fs::read_to_string(&portal.path).unwrap();
    assert!(text.contains("id = \"first\"") && !text.contains("id = \"second\""));
}

#[tokio::test]
async fn invalid_fields_are_listed_and_nothing_is_written() {
    let portal = portal();
    let before = fs::read(&portal.path).unwrap();
    let current = revision(&portal.router).await;
    let (status, _, body) = send(
        &portal.router,
        write(
            "POST",
            ServicesFeature::COLLECTION,
            Some(&current),
            r#"{"id":"Bad Id","name":"X","url":"ftp://x"}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let fields: Vec<&str> = body["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|error| error["field"].as_str().unwrap())
        .collect();
    assert_eq!(fields, vec!["id", "url"]);
    assert_eq!(fs::read(&portal.path).unwrap(), before);
}

#[tokio::test]
async fn a_duplicate_id_is_reported_on_the_id_field() {
    let portal = portal();
    let current = revision(&portal.router).await;
    let (status, _, body) = send(
        &portal.router,
        write(
            "POST",
            ServicesFeature::COLLECTION,
            Some(&current),
            &service_json("a-second", "http://x"),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["errors"][0]["field"], "id");
}

#[tokio::test]
async fn editing_renames_and_deleting_an_unknown_service_is_not_found() {
    let portal = portal();
    let current = revision(&portal.router).await;
    let (status, _, body) = send(
        &portal.router,
        write(
            "PUT",
            "/api/services/b-first",
            Some(&current),
            &service_json("renamed", "http://10.255.0.2"),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(
        fs::read_to_string(&portal.path)
            .unwrap()
            .contains("id = \"renamed\"")
    );
    let current = revision(&portal.router).await;
    let (status, _, _) = send(
        &portal.router,
        write("DELETE", "/api/services/nope", Some(&current), ""),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn deleting_a_service_answers_with_the_remaining_list() {
    let portal = portal();
    let current = revision(&portal.router).await;
    let (status, _, body) = send(
        &portal.router,
        write("DELETE", "/api/services/b-first", Some(&current), ""),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["services"].as_array().unwrap().len(), 1);
    assert!(
        !fs::read_to_string(&portal.path)
            .unwrap()
            .contains("b-first")
    );
}

const WITH_ENVIRONMENTS: &str = r#"[environments.local]
networks = ["192.168.0.0/16"]

[[services]]
id = "nas"
name = "NAS"
url = "http://nas.local"
addresses = { local = "http://192.168.1.10", internet = "https://nas.example.com" }
probe = { enabled = false }

[[services]]
id = "printer"
name = "Printer"
url = "http://printer.local"
environments = ["local"]
probe = { enabled = false }
"#;

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
        path,
        _directory: directory,
    }
}

async fn listed(portal: &Portal, environment: &str) -> Value {
    let request = Request::get(ServicesFeature::COLLECTION)
        .body(Body::empty())
        .unwrap();
    let environment = Environment::parse(environment).unwrap();
    send_from(&portal.router, request, environment).await.2
}

#[tokio::test]
async fn a_visitor_sees_the_address_of_their_own_environment() {
    let portal = portal_with(WITH_ENVIRONMENTS);
    let local = listed(&portal, "local").await;
    assert_eq!(local["services"][0]["address"], "http://192.168.1.10");
    assert_eq!(local["services"][0]["url"], "http://nas.local");
    let outside = listed(&portal, "internet").await;
    assert_eq!(outside["services"][0]["address"], "https://nas.example.com");
}

#[tokio::test]
async fn a_service_of_another_environment_is_absent() {
    let portal = portal_with(WITH_ENVIRONMENTS);
    let local = listed(&portal, "local").await;
    let ids: Vec<&str> = local["services"]
        .as_array()
        .unwrap()
        .iter()
        .map(|service| service["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, vec!["nas", "printer"]);
    let outside = listed(&portal, "internet").await;
    let ids: Vec<&str> = outside["services"]
        .as_array()
        .unwrap()
        .iter()
        .map(|service| service["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, vec!["nas"]);
}

#[tokio::test]
async fn a_service_falls_back_to_its_url_where_it_has_no_address() {
    let portal = portal_with(WITH_ENVIRONMENTS);
    let local = listed(&portal, "local").await;
    assert_eq!(local["services"][1]["address"], "http://printer.local");
}
