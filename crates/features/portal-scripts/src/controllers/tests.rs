use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use portal_config::ConfigStore;
use portal_feature::Feature;
use portal_model::{DetectedEnvironment, Environment, Environments};
use serde_json::{Value, json};
use tower::ServiceExt;

use crate::fakes::FileOwner;
use crate::features::ScriptsFeature;

struct Api {
    folder: tempfile::TempDir,
    router: Router,
}

fn api(configuration: &str) -> Api {
    let folder = tempfile::tempdir().unwrap();
    let main = folder.path().join("home-portal.toml");
    fs::write(&main, configuration).unwrap();
    let root = folder.path().join("scripts");
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o755)).unwrap();
    let store = Arc::new(ConfigStore::open(&main).unwrap());
    let feature = ScriptsFeature::new(store, Arc::new(FileOwner::of_this_process()));
    Api {
        router: feature.router(),
        folder,
    }
}

const ON: &str = "[scripts]\nediting = true\n";

fn from(environment: &str) -> DetectedEnvironment {
    DetectedEnvironment::new(
        Environment::parse(environment).unwrap(),
        &Environments::default(),
    )
}

async fn send(
    api: &Api,
    mut request: Request<Body>,
    environment: &str,
) -> (StatusCode, Option<String>, Value) {
    request.extensions_mut().insert(from(environment));
    let response = api.router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let etag = response
        .headers()
        .get("etag")
        .map(|value| value.to_str().unwrap().to_string());
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = serde_json::from_slice(&bytes)
        .unwrap_or(Value::String(String::from_utf8_lossy(&bytes).to_string()));
    (status, etag, body)
}

fn json_request(method: &str, uri: &str, body: Value, revision: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");
    if let Some(revision) = revision {
        builder = builder.header("if-match", revision);
    }
    builder.body(Body::from(body.to_string())).unwrap()
}

fn get(uri: &str) -> Request<Body> {
    Request::get(uri).body(Body::empty()).unwrap()
}

#[tokio::test]
async fn every_endpoint_is_absent_while_editing_is_off() {
    let api = api("");
    fs::write(api.folder.path().join("scripts/backup.sh"), "keep\n").unwrap();
    let (status, _, _) = send(&api, get(ScriptsFeature::TREE), "local").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _, _) = send(
        &api,
        json_request(
            "PUT",
            "/api/scripts/file?path=backup.sh",
            json!({"content": "x"}),
            Some("\"r\""),
        ),
        "local",
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(
        fs::read_to_string(api.folder.path().join("scripts/backup.sh")).unwrap(),
        "keep\n"
    );
}

#[tokio::test]
async fn a_script_is_created_read_saved_and_its_entry_comes_back_after_each_write() {
    let api = api(ON);
    fs::create_dir(api.folder.path().join("scripts/media")).unwrap();
    fs::set_permissions(
        api.folder.path().join("scripts/media"),
        fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    let (status, created, entry) = send(
        &api,
        json_request(
            "POST",
            ScriptsFeature::FILE,
            json!({"path": "media/restart.sh", "content": "#!/bin/sh\n# @arg service <text> Service id\n"}),
            None,
        ),
        "local",
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(entry["runnable"], true);
    assert_eq!(entry["mode"], "0700");
    assert_eq!(entry["arguments"][0]["name"], "service");
    let (status, etag, body) = send(
        &api,
        get("/api/scripts/file?path=media/restart.sh"),
        "local",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(etag, created);
    assert!(body["content"].as_str().unwrap().contains("@arg service"));
    let (status, saved, entry) = send(
        &api,
        json_request(
            "PUT",
            "/api/scripts/file?path=media/restart.sh",
            json!({"content": "#!/bin/sh\nexit 0\n"}),
            etag.as_deref(),
        ),
        "local",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_ne!(saved, etag);
    assert_eq!(entry["arguments"], json!([]));
    let (status, _, _) = send(
        &api,
        json_request(
            "PUT",
            "/api/scripts/file?path=media/restart.sh",
            json!({"content": "late\n"}),
            etag.as_deref(),
        ),
        "local",
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, _, tree) = send(&api, get(ScriptsFeature::TREE), "local").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(tree["folders"], json!(["media"]));
    assert_eq!(tree["files"][0]["path"], "media/restart.sh");
    assert_eq!(tree["inside"], true);
}

#[tokio::test]
async fn a_write_from_the_internet_is_forbidden_and_a_read_is_not() {
    let api = api(ON);
    fs::write(api.folder.path().join("scripts/backup.sh"), "keep\n").unwrap();
    let (status, etag, _) = send(&api, get("/api/scripts/file?path=backup.sh"), "internet").await;
    assert_eq!(status, StatusCode::OK);
    let (status, _, body) = send(
        &api,
        json_request(
            "PUT",
            "/api/scripts/file?path=backup.sh",
            json!({"content": "changed\n"}),
            etag.as_deref(),
        ),
        "internet",
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(body.as_str().unwrap().contains("only from inside"));
    assert_eq!(
        fs::read_to_string(api.folder.path().join("scripts/backup.sh")).unwrap(),
        "keep\n"
    );
    let (_, _, tree) = send(&api, get(ScriptsFeature::TREE), "internet").await;
    assert_eq!(tree["inside"], false);
}

#[tokio::test]
async fn a_path_that_climbs_out_or_a_write_without_a_revision_is_refused() {
    let api = api(ON);
    let (status, _, body) = send(
        &api,
        json_request(
            "POST",
            ScriptsFeature::FILE,
            json!({"path": "../home-portal.toml", "content": "x"}),
            None,
        ),
        "local",
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["errors"][0]["field"], "path");
    fs::write(api.folder.path().join("scripts/backup.sh"), "keep\n").unwrap();
    let (status, _, _) = send(
        &api,
        json_request(
            "PUT",
            "/api/scripts/file?path=backup.sh",
            json!({"content": "x"}),
            None,
        ),
        "local",
    )
    .await;
    assert_eq!(status, StatusCode::PRECONDITION_REQUIRED);
}

#[tokio::test]
async fn folders_are_created_moved_into_and_removed_only_when_empty() {
    let api = api(ON);
    let (status, _, _) = send(
        &api,
        json_request(
            "POST",
            ScriptsFeature::FOLDER,
            json!({"name": "media"}),
            None,
        ),
        "local",
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (_, revision, _) = send(
        &api,
        json_request(
            "POST",
            ScriptsFeature::FILE,
            json!({"path": "restart.sh", "content": "x\n"}),
            None,
        ),
        "local",
    )
    .await;
    let (status, _, entry) = send(
        &api,
        json_request(
            "POST",
            ScriptsFeature::MOVE,
            json!({"from": "restart.sh", "to": "media/restart.sh"}),
            revision.as_deref(),
        ),
        "local",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(entry["path"], "media/restart.sh");
    let request = Request::delete("/api/scripts/folder?name=media")
        .body(Body::empty())
        .unwrap();
    let (status, _, body) = send(&api, request, "local").await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(body.as_str().unwrap().contains("restart.sh"));
    let request = Request::delete("/api/scripts/file?path=media/restart.sh")
        .header("if-match", revision.clone().unwrap())
        .body(Body::empty())
        .unwrap();
    let (status, _, _) = send(&api, request, "local").await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let request = Request::delete("/api/scripts/folder?name=media")
        .body(Body::empty())
        .unwrap();
    let (status, _, _) = send(&api, request, "local").await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn a_missing_directory_is_an_empty_tree_and_can_be_created() {
    let api = api(ON);
    fs::remove_dir(api.folder.path().join("scripts")).unwrap();
    let (status, _, tree) = send(&api, get(ScriptsFeature::TREE), "local").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(tree["exists"], false);
    let (status, _, _) = send(
        &api,
        json_request("POST", ScriptsFeature::FOLDER, json!({}), None),
        "local",
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let mode = fs::metadata(api.folder.path().join("scripts"))
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o700);
}
