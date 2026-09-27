use std::fs;

use axum::body::Body;
use axum::http::header::CONTENT_TYPE;
use axum::http::{Request, StatusCode};

use super::automations::{api, send, write};
use crate::features::tests::{eventually, start, write_script};

const OFF: &str = "[modules]\nautomations = false\nwebhooks = false\n\n";
const WEBHOOKS_OFF: &str = "[modules]\nwebhooks = false\n\n";
const HOOK: &str = "0b9e8c2a-1d3f-4a5b-8c7d-9e0f1a2b3c4d";

const BACKUP: &str = r#"[[automations]]
id = "backup"
title = "Backup"
when = { event = "manual" }
run = { script = "backup.sh" }
"#;

fn hook() -> String {
    format!(
        "[[webhooks]]\nid = \"{HOOK}\"\ntitle = \"Motion\"\naction = \"script\"\nrun = {{ script = \"backup.sh\" }}\n"
    )
}

fn call() -> Request<Body> {
    Request::post(format!("/webhook/{HOOK}"))
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from("{}"))
        .unwrap()
}

#[tokio::test]
async fn run_now_is_refused_while_automations_are_off() {
    let api = api(&format!("{OFF}{BACKUP}"));
    write_script(&api.folder.path().join("scripts"), "backup.sh", "true");
    start(&api.feature);
    let (status, _, body) =
        send(&api, write("POST", "/api/automations/backup/run", None, "")).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(
        body.as_str().unwrap_or_default().contains("module is off"),
        "{body}"
    );
    assert!(api.feature.state.sink.journal.snapshot().is_empty());
}

#[tokio::test]
async fn automations_can_still_be_read_while_off() {
    let api = api(&format!("{OFF}{BACKUP}"));
    let (status, _, body) = send(&api, super::automations::get("/api/automations")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["automations"][0]["id"], "backup");
}

#[tokio::test]
async fn a_webhook_is_not_found_while_webhooks_are_off() {
    let api = api(&format!("{WEBHOOKS_OFF}{}", hook()));
    write_script(&api.folder.path().join("scripts"), "backup.sh", "true");
    start(&api.feature);
    let (status, _, _) = send(&api, call()).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(api.feature.state.sink.journal.snapshot().is_empty());
}

#[tokio::test]
async fn webhooks_answer_again_within_five_seconds_of_switching_on() {
    let api = api(&format!("{WEBHOOKS_OFF}{}", hook()));
    write_script(&api.folder.path().join("scripts"), "backup.sh", "true");
    start(&api.feature);
    let (status, _, _) = send(&api, call()).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    fs::write(api.folder.path().join("home-portal.toml"), hook()).unwrap();
    let cache = api.feature.state.sink.cache.clone();
    let began = std::time::Instant::now();
    assert!(eventually(|| cache.webhooks_on()).await);
    assert!(began.elapsed() < std::time::Duration::from_secs(5));
    let (status, _, _) = send(&api, call()).await;
    assert_eq!(status, StatusCode::ACCEPTED);
}
