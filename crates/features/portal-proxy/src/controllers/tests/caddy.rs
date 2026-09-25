use std::fs;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::Value;

use super::support::{CADDY, Portal, file, get, json, portal, portal_pinned, portal_with, send};
use crate::types::CaddyHome;

async fn revision_of(portal: &Portal) -> String {
    let response = send(portal, get("/api/proxy"), CADDY, "local").await;
    response.headers()["etag"].to_str().unwrap().to_string()
}

fn post(uri: &str, revision: Option<&str>) -> Request<Body> {
    let mut builder = Request::post(uri);
    if let Some(revision) = revision {
        builder = builder.header("if-match", revision);
    }
    builder.body(Body::empty()).unwrap()
}

async fn caddy_block(portal: &Portal) -> Value {
    json(send(portal, get("/api/proxy"), CADDY, "local").await).await["caddy"].clone()
}

#[tokio::test]
async fn downloading_answers_at_once_and_reports_the_installed_version() {
    let releases = crate::fakes::Releases::serve(
        crate::fakes::Releases::archive_of(crate::fakes::SCRIPT),
        false,
        Some(std::time::Duration::from_millis(200)),
    )
    .await;
    let portal = portal_with(&file("http://127.0.0.1:9"), &releases.base);
    let response = send(
        &portal,
        post("/api/proxy/caddy/download", None),
        CADDY,
        "local",
    )
    .await;
    assert_eq!(response.status(), StatusCode::ACCEPTED);
    assert_eq!(
        json(response).await["caddy"]["download"]["state"],
        "downloading"
    );
    let again = send(
        &portal,
        post("/api/proxy/caddy/download", None),
        CADDY,
        "local",
    )
    .await;
    assert_eq!(again.status(), StatusCode::CONFLICT);
    let mut installed = Value::Null;
    for _ in 0..100 {
        installed = caddy_block(&portal).await;
        if installed["download"]["state"] != "downloading" {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert_eq!(installed["download"]["state"], "installed");
    assert_eq!(installed["installed"], crate::fakes::VERSION);
}

#[tokio::test]
async fn starting_without_a_downloaded_caddy_is_refused() {
    let portal = portal(&file("http://127.0.0.1:9"));
    let revision = revision_of(&portal).await;
    let response = send(
        &portal,
        post("/api/proxy/caddy/start", Some(&revision)),
        CADDY,
        "local",
    )
    .await;
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[cfg(unix)]
#[tokio::test]
async fn start_and_stop_write_managed_and_need_the_revision() {
    use std::os::unix::fs::PermissionsExt;
    let portal = portal(&file("http://127.0.0.1:9"));
    let home = CaddyHome::at(portal.path.with_file_name("caddy"));
    fs::create_dir_all(&home.directory).unwrap();
    fs::write(home.binary(), crate::fakes::SCRIPT).unwrap();
    fs::set_permissions(home.binary(), fs::Permissions::from_mode(0o755)).unwrap();
    let without = send(
        &portal,
        post("/api/proxy/caddy/start", None),
        CADDY,
        "local",
    )
    .await;
    assert_eq!(without.status(), StatusCode::PRECONDITION_REQUIRED);
    let revision = revision_of(&portal).await;
    let started = send(
        &portal,
        post("/api/proxy/caddy/start", Some(&revision)),
        CADDY,
        "local",
    )
    .await;
    assert_eq!(started.status(), StatusCode::OK);
    assert_eq!(json(started).await["caddy"]["managed"], true);
    assert!(
        fs::read_to_string(&portal.path)
            .unwrap()
            .contains("managed = true")
    );
    let record = home.directory.join("launches.txt");
    for _ in 0..100 {
        if record.exists() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    let launched = fs::read_to_string(&record).unwrap();
    let revision = revision_of(&portal).await;
    let stopped = send(
        &portal,
        post("/api/proxy/caddy/stop", Some(&revision)),
        CADDY,
        "local",
    )
    .await;
    assert_eq!(stopped.status(), StatusCode::OK);
    assert!(
        fs::read_to_string(&portal.path)
            .unwrap()
            .contains("managed = false")
    );
    if let Some(pid) = launched.split_whitespace().next() {
        let _ = std::process::Command::new("kill").arg(pid).status();
    }
}

async fn settled(portal: &Portal) -> Value {
    let mut block = Value::Null;
    for _ in 0..100 {
        block = caddy_block(portal).await;
        if block["download"]["state"] != "downloading" {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    block
}

#[tokio::test]
async fn a_pinned_version_is_downloaded_from_the_configured_source() {
    let releases = crate::fakes::Releases::serve(
        crate::fakes::Releases::archive_of(crate::fakes::SCRIPT),
        false,
        None,
    )
    .await;
    let portal = portal_pinned(
        &file("http://127.0.0.1:9"),
        &releases.base,
        crate::fakes::VERSION,
    );
    let response = send(
        &portal,
        post("/api/proxy/caddy/download", None),
        CADDY,
        "local",
    )
    .await;
    assert_eq!(response.status(), StatusCode::ACCEPTED);
    let block = settled(&portal).await;
    assert_eq!(block["installed"], crate::fakes::VERSION);
    assert_eq!(
        block["installed_from"],
        format!("{}/archive", releases.base)
    );
}

#[tokio::test]
async fn a_version_that_does_not_exist_fails_the_download_naming_the_url() {
    let releases = crate::fakes::Releases::serve(
        crate::fakes::Releases::archive_of(crate::fakes::SCRIPT),
        false,
        None,
    )
    .await;
    let portal = portal_pinned(&file("http://127.0.0.1:9"), &releases.base, "1.2.3");
    send(
        &portal,
        post("/api/proxy/caddy/download", None),
        CADDY,
        "local",
    )
    .await;
    let block = settled(&portal).await;
    assert_eq!(block["download"]["state"], "failed");
    let error = block["download"]["error"].as_str().unwrap();
    assert!(
        error.contains(&format!("{}/tags/v1.2.3", releases.base)),
        "{error}"
    );
    assert_eq!(block["installed"], Value::Null);
}

fn put_source(body: &str, revision: Option<&str>) -> Request<Body> {
    let mut builder = Request::put("/api/proxy/caddy").header("content-type", "application/json");
    if let Some(revision) = revision {
        builder = builder.header("if-match", revision);
    }
    builder.body(Body::from(body.to_string())).unwrap()
}

#[tokio::test]
async fn pinning_a_version_writes_it_and_answers_with_the_new_revision() {
    let portal = portal(&file("http://127.0.0.1:9"));
    let revision = revision_of(&portal).await;
    let before = fs::read_to_string(&portal.path).unwrap();
    let response = send(
        &portal,
        put_source(r#"{"source":"","version":"2.10.2"}"#, Some(&revision)),
        CADDY,
        "local",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let etag = response.headers()["etag"].to_str().unwrap().to_string();
    assert_ne!(etag, revision);
    let body = json(response).await;
    assert_eq!(body["caddy"]["version"], "2.10.2");
    assert!(
        body["caddy"]["release_url"]
            .as_str()
            .unwrap()
            .ends_with("/tags/v2.10.2")
    );
    let after = fs::read_to_string(&portal.path).unwrap();
    assert!(after.contains(r#"version = "2.10.2""#), "{after}");
    assert_eq!(after.lines().count(), before.lines().count(), "{after}");
}

#[tokio::test]
async fn a_malformed_version_is_refused_by_name_and_the_file_is_unchanged() {
    let portal = portal(&file("http://127.0.0.1:9"));
    let revision = revision_of(&portal).await;
    let before = fs::read_to_string(&portal.path).unwrap();
    let response = send(
        &portal,
        put_source(r#"{"version":"newest"}"#, Some(&revision)),
        CADDY,
        "local",
    )
    .await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = json(response).await;
    assert!(body.to_string().contains("proxy.caddy.version"), "{body}");
    assert_eq!(fs::read_to_string(&portal.path).unwrap(), before);
}

#[tokio::test]
async fn changing_the_source_needs_the_revision() {
    let portal = portal(&file("http://127.0.0.1:9"));
    let response = send(
        &portal,
        put_source(r#"{"version":"2.10.2"}"#, None),
        CADDY,
        "local",
    )
    .await;
    assert_eq!(response.status(), StatusCode::PRECONDITION_REQUIRED);
}
