use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::header::{CONTENT_TYPE, ETAG, IF_MATCH};
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use portal_config::ConfigStore;
use portal_feature::{Feature, FieldError};
use portal_model::Environment;
use serde_json::{Value, json};
use tempfile::TempDir;
use toml_edit::DocumentMut;
use tower::ServiceExt;

use crate::DashboardFeature;

const FILE: &str = "# my portal\n\n[network]\nport = 8080 # keep\n\n# counts\n[[dashboard.widgets]]\ntype = \"status-summary\"\n\n[[dashboard.widgets]]\ntype = \"weather\"\nid = \"riga\"\nsettings = { latitude = 56.95 }\n";

struct Portal {
    router: Router,
    path: PathBuf,
    _directory: TempDir,
}

fn latitude_check(document: &DocumentMut) -> Vec<FieldError> {
    portal_widget::WidgetsSection::read(document)
        .unwrap_or_default()
        .into_iter()
        .filter(|widget| {
            widget.settings["latitude"]
                .as_f64()
                .is_some_and(|latitude| latitude > 90.0)
        })
        .map(|widget| {
            FieldError::new(
                format!(
                    "dashboard.widgets.{}.settings.latitude",
                    widget.id.unwrap_or_default()
                ),
                "must be between -90 and 90",
            )
        })
        .collect()
}

fn portal_with(files: &[(&str, &str)]) -> Portal {
    let directory = tempfile::tempdir().unwrap();
    for (name, text) in files {
        fs::write(directory.path().join(name), text).unwrap();
    }
    let path = directory.path().join(files[0].0);
    let store = Arc::new(ConfigStore::open(&path).unwrap());
    let feature = DashboardFeature::new(store.clone());
    store.adopt(vec![feature.validator().unwrap()]).unwrap();
    store.adopt_checks(vec![Arc::new(latitude_check)]).unwrap();
    Portal {
        router: feature.router(),
        path,
        _directory: directory,
    }
}

async fn send(portal: &Portal, request: Request<Body>) -> (StatusCode, Option<String>, Value) {
    let mut request = request;
    request.extensions_mut().insert(Environment::internet());
    let response = portal.router.clone().oneshot(request).await.unwrap();
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

async fn loaded(portal: &Portal) -> (String, Value) {
    let (_, etag, body) = send(
        portal,
        Request::get("/api/dashboard?all=true")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    (etag.unwrap(), body)
}

fn put(revision: Option<&str>, body: &Value) -> Request<Body> {
    let mut request = Request::put(DashboardFeature::PATH).header(CONTENT_TYPE, "application/json");
    if let Some(revision) = revision {
        request = request.header(IF_MATCH, revision);
    }
    request.body(Body::from(body.to_string())).unwrap()
}

fn reversed(body: &Value) -> Value {
    let mut widgets = body["widgets"].as_array().unwrap().clone();
    widgets.reverse();
    json!({ "sections": body["sections"], "widgets": widgets })
}

#[tokio::test]
async fn a_saved_layout_is_written_and_the_rest_of_the_file_is_byte_identical() {
    let portal = portal_with(&[("home-portal.toml", FILE)]);
    let (revision, body) = loaded(&portal).await;
    let (status, etag, saved) = send(&portal, put(Some(&revision), &reversed(&body))).await;
    assert_eq!(status, StatusCode::OK, "{saved}");
    assert_ne!(etag.unwrap(), revision);
    assert_eq!(saved["widgets"][0]["key"], "riga");
    assert_eq!(saved["widgets"][1]["key"], "status-summary");
    let text = fs::read_to_string(&portal.path).unwrap();
    assert!(
        text.starts_with("# my portal\n\n[network]\nport = 8080 # keep\n"),
        "{text}"
    );
    assert!(
        text.ends_with(
            "# counts\n[[dashboard.widgets]]\ntype = \"status-summary\"\nid = \"status-summary\"\n"
        ),
        "{text}"
    );
}

#[tokio::test]
async fn a_save_without_a_revision_is_428_and_with_a_stale_one_is_409() {
    let portal = portal_with(&[("home-portal.toml", FILE)]);
    let (revision, body) = loaded(&portal).await;
    assert_eq!(
        send(&portal, put(None, &body)).await.0,
        StatusCode::PRECONDITION_REQUIRED
    );
    fs::write(&portal.path, format!("{FILE}\n# edited by hand\n")).unwrap();
    let file = fs::File::options().write(true).open(&portal.path).unwrap();
    file.set_modified(std::time::SystemTime::now() + std::time::Duration::from_secs(5))
        .unwrap();
    assert_eq!(
        send(&portal, put(Some(&revision), &body)).await.0,
        StatusCode::CONFLICT
    );
}

#[tokio::test]
async fn invalid_widgets_are_named_by_their_place_in_the_editor() {
    let portal = portal_with(&[("home-portal.toml", FILE)]);
    let (revision, body) = loaded(&portal).await;
    let mut bad = body.clone();
    bad["widgets"][0]["size"] = json!("wide");
    bad["widgets"][0]["section"] = json!("nowhere");
    let (status, _, errors) = send(&portal, put(Some(&revision), &bad)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let fields: Vec<&str> = errors["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|error| error["field"].as_str().unwrap())
        .collect();
    assert_eq!(fields, vec!["widgets[0].size", "widgets[0].section"]);
    let mut far = body.clone();
    far["widgets"][1]["settings"] = json!({ "latitude": 120.0 });
    let (status, _, errors) = send(&portal, put(Some(&revision), &far)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(errors["errors"][0]["field"], "widgets[1].settings.latitude");
    assert_eq!(fs::read_to_string(&portal.path).unwrap(), FILE);
}

#[tokio::test]
async fn a_layout_spread_over_two_files_is_refused_naming_both_and_nothing_changes() {
    let main = format!("include = [\"widgets.toml\"]\n\n{FILE}");
    let extra = "[[dashboard.widgets]]\ntype = \"services\"\n";
    let portal = portal_with(&[("home-portal.toml", &main), ("widgets.toml", extra)]);
    let (revision, body) = loaded(&portal).await;
    let (status, _, message) = send(&portal, put(Some(&revision), &reversed(&body))).await;
    assert_eq!(status, StatusCode::CONFLICT);
    let message = message.as_str().unwrap();
    assert!(
        message.contains("home-portal.toml") && message.contains("widgets.toml"),
        "{message}"
    );
    assert_eq!(fs::read_to_string(&portal.path).unwrap(), main);
}
