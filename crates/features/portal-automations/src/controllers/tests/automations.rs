use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::header::{CONTENT_TYPE, ETAG, IF_MATCH};
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use portal_feature::{Feature, Principal};
use serde_json::Value;
use tempfile::TempDir;
use tower::ServiceExt;

use crate::features::AutomationsFeature;
use crate::features::tests::{eventually, portal, start};

const FILE: &str = r#"[automation_settings]
timezone = "Europe/Berlin"

# Restarts the media server.
[[automations]]
id = "restart-media"
title = "Restart"
cooldown_seconds = 300 # five minutes
when = { event = "service.status-changed", services = ["jellyfin"], to = ["down"] }
run = { script = "restart.sh", args = ["--", "{{service.id}}"] }

[[automations]]
id = "sleeping"
title = "Sleeping"
enabled = false
when = { event = "portal.started" }
run = { script = "restart.sh" }
"#;

pub struct Api {
    pub router: Router,
    pub feature: AutomationsFeature,
    pub folder: TempDir,
}

pub fn api(text: &str) -> Api {
    let (folder, feature) = portal(text, &[("restart.sh", "echo \"$@\"")]);
    feature
        .state
        .configuration
        .adopt(vec![feature.validator().unwrap()])
        .unwrap();
    let router = feature
        .router()
        .layer(axum::Extension(Principal {
            name: "admin".into(),
        }))
        .merge(feature.public_router());
    Api {
        router,
        feature,
        folder,
    }
}

pub async fn send(api: &Api, request: Request<Body>) -> (StatusCode, Option<String>, Value) {
    let response = api.router.clone().oneshot(request).await.unwrap();
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

pub fn get(uri: &str) -> Request<Body> {
    Request::get(uri).body(Body::empty()).unwrap()
}

pub fn write(method: &str, uri: &str, revision: Option<&str>, body: &str) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(CONTENT_TYPE, "application/json");
    if let Some(revision) = revision {
        builder = builder.header(IF_MATCH, revision);
    }
    builder.body(Body::from(body.to_string())).unwrap()
}

pub async fn revision(api: &Api) -> String {
    send(api, get(AutomationsFeature::COLLECTION))
        .await
        .1
        .unwrap()
}

const NEW: &str = r#"{"id":"nightly","title":"Nightly","when":{"event":"schedule","cron":"0 3 * * *"},"run":{"script":"restart.sh","args":["{{schedule.at}}"]}}"#;

#[tokio::test]
async fn the_list_carries_every_automation_with_its_revision() {
    let api = api(FILE);
    let (status, etag, body) = send(&api, get(AutomationsFeature::COLLECTION)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(etag.is_some());
    assert_eq!(body["automations"][0]["id"], "restart-media");
    assert_eq!(body["automations"][0]["when"]["services"][0], "jellyfin");
    assert_eq!(body["automations"][0]["run"]["timeout_seconds"], 60);
    assert_eq!(body["automations"][0]["last_run"], Value::Null);
    assert_eq!(body["automations"][1]["enabled"], false);
}

#[tokio::test]
async fn a_write_needs_the_current_revision() {
    let api = api(FILE);
    let (status, _, _) = send(
        &api,
        write("POST", AutomationsFeature::COLLECTION, None, NEW),
    )
    .await;
    assert_eq!(status, StatusCode::PRECONDITION_REQUIRED);
    let (status, _, _) = send(
        &api,
        write(
            "POST",
            AutomationsFeature::COLLECTION,
            Some("\"stale\""),
            NEW,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    let current = revision(&api).await;
    let (status, etag, body) = send(
        &api,
        write("POST", AutomationsFeature::COLLECTION, Some(&current), NEW),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_ne!(etag.unwrap(), current);
    assert_eq!(body["when"]["cron"], "0 3 * * *");
    let text = fs::read_to_string(api.folder.path().join("home-portal.toml")).unwrap();
    assert!(text.contains("id = \"nightly\""));
}

#[tokio::test]
async fn invalid_fields_are_refused_one_by_one_and_nothing_is_written() {
    let api = api(FILE);
    let before = fs::read_to_string(api.folder.path().join("home-portal.toml")).unwrap();
    let current = revision(&api).await;
    let body = r#"{"id":"bad","title":"Bad","when":{"event":"portal.started","to":["down"]},"run":{"script":"../home-portal.toml","args":["{{service.id}}"]}}"#;
    let (status, _, answer) = send(
        &api,
        write("POST", AutomationsFeature::COLLECTION, Some(&current), body),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let fields: Vec<&str> = answer["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|error| error["field"].as_str().unwrap())
        .collect();
    assert_eq!(fields, vec!["when.to", "run.script"]);
    let (status, _, answer) = send(
        &api,
        write(
            "POST",
            AutomationsFeature::COLLECTION,
            Some(&current),
            r#"{"id":"bad","title":"Bad","when":{"event":"portal.started"},"run":{"script":"a.sh","args":["{{service.id}}"]}}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(answer["errors"][0]["field"], "run.args[0]");
    let after = fs::read_to_string(api.folder.path().join("home-portal.toml")).unwrap();
    assert_eq!(before, after);
}

#[tokio::test]
async fn an_edit_keeps_the_comments_and_the_untouched_keys() {
    let api = api(FILE);
    let current = revision(&api).await;
    let body = r#"{"id":"restart-media","title":"Restart Jellyfin","cooldown_seconds":300,"when":{"event":"service.status-changed","services":["jellyfin"],"to":["down"]},"run":{"script":"restart.sh","args":["--","{{service.id}}"]}}"#;
    let (status, _, _) = send(
        &api,
        write(
            "PUT",
            "/api/automations/restart-media",
            Some(&current),
            body,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let text = fs::read_to_string(api.folder.path().join("home-portal.toml")).unwrap();
    assert_eq!(
        text,
        FILE.replace("title = \"Restart\"", "title = \"Restart Jellyfin\"")
    );
}

#[tokio::test]
async fn an_automation_in_an_included_file_is_edited_in_that_file() {
    let folder = TempDir::new().unwrap();
    let included = folder.path().join("automations.toml");
    fs::write(&included, FILE).unwrap();
    let main = folder.path().join("home-portal.toml");
    fs::write(&main, "include = [\"automations.toml\"]\n").unwrap();
    for path in [&main, &included] {
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
    }
    let store = Arc::new(portal_config::ConfigStore::open(&main).unwrap());
    let feature = AutomationsFeature::new(store, Arc::new(crate::features::tests::FakeDirectory));
    let api = Api {
        router: feature.router(),
        feature,
        folder,
    };
    let current = revision(&api).await;
    let (status, _, _) = send(
        &api,
        write("DELETE", "/api/automations/sleeping", Some(&current), ""),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let text = fs::read_to_string(&included).unwrap();
    assert!(!text.contains("sleeping"));
    assert!(text.contains("restart-media"));
    let main = fs::read_to_string(api.folder.path().join("home-portal.toml")).unwrap();
    assert_eq!(main, "include = [\"automations.toml\"]\n");
}

#[tokio::test]
async fn the_catalogue_lists_events_fields_samples_filters_and_choices() {
    let api = api(FILE);
    let (status, _, body) = send(&api, get(AutomationsFeature::CATALOGUE)).await;
    assert_eq!(status, StatusCode::OK);
    let events = body["events"].as_array().unwrap();
    let changed = events
        .iter()
        .find(|event| event["name"] == "service.status-changed")
        .unwrap();
    let fields: Vec<&str> = changed["fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|field| field["name"].as_str().unwrap())
        .collect();
    assert!(fields.contains(&"service.id") && fields.contains(&"status.diagnosis"));
    assert!(
        changed["fields"]
            .as_array()
            .unwrap()
            .iter()
            .all(|field| field["sample"].is_string())
    );
    assert_eq!(
        changed["filters"],
        serde_json::json!(["services", "from", "to", "from_unknown"])
    );
    assert_eq!(body["states"].as_array().unwrap().len(), 5);
    assert_eq!(body["choices"]["services"][0]["id"], "nas");
    assert_eq!(body["choices"]["environments"][1], "internet");
}

#[tokio::test]
async fn the_scripts_say_whether_each_can_run_and_a_missing_directory_is_empty() {
    let api = api(FILE);
    let open = api.folder.path().join("scripts/open.sh");
    fs::write(&open, "#!/bin/sh\n").unwrap();
    fs::set_permissions(&open, fs::Permissions::from_mode(0o777)).unwrap();
    let (status, _, body) = send(&api, get(AutomationsFeature::SCRIPTS)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["exists"], true);
    assert_eq!(body["scripts"][0]["path"], "open.sh");
    assert_eq!(body["scripts"][0]["runnable"], false);
    assert!(
        body["scripts"][0]["problem"]
            .as_str()
            .unwrap()
            .contains("others")
    );
    assert_eq!(body["scripts"][1]["runnable"], true);
    fs::remove_dir_all(api.folder.path().join("scripts")).unwrap();
    let (_, _, body) = send(&api, get(AutomationsFeature::SCRIPTS)).await;
    assert_eq!(body["exists"], false);
    assert_eq!(body["scripts"], serde_json::json!([]));
}

#[tokio::test]
async fn the_schedule_lists_five_times_with_their_offsets_or_names_the_problem() {
    let api = api(FILE);
    let (status, _, body) = send(
        &api,
        get("/api/automations/schedule?cron=0%203%20*%20*%20mon-fri"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["timezone"], "Europe/Berlin");
    let times = body["times"].as_array().unwrap();
    assert_eq!(times.len(), 5);
    assert!(times.iter().all(|time| {
        let text = time.as_str().unwrap();
        text.contains("T03:00:00+01:00") || text.contains("T03:00:00+02:00")
    }));
    let (status, _, body) = send(
        &api,
        get("/api/automations/schedule?cron=61%20*%20*%20*%20*"),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["errors"][0]["field"], "cron");
    assert!(
        body["errors"][0]["message"]
            .as_str()
            .unwrap()
            .contains("minute must be 0 to 59")
    );
    let (status, _, body) = send(
        &api,
        get("/api/automations/schedule?cron=0%200%2030%202%20*"),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(
        body["errors"][0]["message"]
            .as_str()
            .unwrap()
            .starts_with("never fires")
    );
}

#[tokio::test]
async fn run_now_queues_a_manual_run_with_samples_and_the_person() {
    let api = api(FILE);
    start(&api.feature);
    let (status, _, body) = send(
        &api,
        write("POST", "/api/automations/restart-media/run", None, ""),
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let run_id = body["run_id"].as_str().unwrap().to_string();
    let sink = api.feature.state.sink.clone();
    assert!(eventually(|| !sink.journal.runs(Some("restart-media")).is_empty()).await);
    let (_, _, body) = send(&api, get("/api/automations/runs?automation=restart-media")).await;
    let run = &body["runs"][0];
    assert_eq!(run["id"], run_id);
    assert_eq!(run["event"], "service.status-changed");
    assert_eq!(run["fields"]["run.manual"], "true");
    assert_eq!(run["fields"]["run.by"], "admin");
    assert_eq!(run["fields"]["service.id"], "jellyfin");
    assert_eq!(run["arguments"], serde_json::json!(["--", "jellyfin"]));
    assert_eq!(run["outcome"]["result"], "succeeded");
    assert_eq!(run["outcome"]["stdout"]["tail"], "-- jellyfin\n");
    let (_, _, listed) = send(&api, get(AutomationsFeature::COLLECTION)).await;
    assert_eq!(listed["automations"][0]["last_run"]["id"], run_id);
    let (status, _, _) = send(
        &api,
        write("POST", "/api/automations/restart-media/run", None, ""),
    )
    .await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn run_now_refuses_a_disabled_or_unknown_automation() {
    let api = api(FILE);
    let (status, _, _) = send(
        &api,
        write("POST", "/api/automations/sleeping/run", None, ""),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, _, _) = send(&api, write("POST", "/api/automations/nope/run", None, "")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
