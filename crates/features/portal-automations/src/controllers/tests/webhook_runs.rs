use axum::body::Body;
use axum::http::header::CONTENT_TYPE;
use axum::http::{Request, StatusCode};

use super::automations::{api, send};
use super::webhooks::{DEPLOY, MOTION, file, ready};
use crate::features::tests::{eventually, start};

fn run(id: &str, body: &str) -> Request<Body> {
    Request::post(format!("/api/webhooks/{id}/run"))
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

#[tokio::test]
async fn running_a_deploy_by_hand_needs_no_token_and_records_who_ran_it() {
    let api = ready();
    start(&api.feature);
    let (status, _, body) = send(&api, run(DEPLOY, r#"{"variables":{"branch":"main"}}"#)).await;
    assert_eq!(status, StatusCode::ACCEPTED, "{body}");
    let run_id = body["run_id"].as_str().unwrap().to_string();
    let sink = api.feature.state.sink.clone();
    assert!(eventually(|| !sink.journal.runs(Some(DEPLOY)).is_empty()).await);
    let record = &sink.journal.runs(Some(DEPLOY))[0];
    assert_eq!(record.id.to_string(), run_id);
    assert_eq!(record.arguments, vec!["--", "main"]);
    assert!(
        record
            .fields
            .iter()
            .any(|(name, value)| name == "run.by" && value == "admin"),
        "{:?}",
        record.fields
    );
    assert!(api.feature.state.webhooks.last(DEPLOY).is_some());
}

#[tokio::test]
async fn an_event_webhook_run_by_hand_is_accepted_without_a_run_id() {
    let api = ready();
    let (status, _, body) = send(&api, run(MOTION, "{}")).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(body["accepted"], true);
    assert_eq!(body["run_id"], serde_json::Value::Null);
}

#[tokio::test]
async fn a_missing_variable_is_named_and_nothing_runs() {
    let api = ready();
    let (status, _, body) = send(&api, run(DEPLOY, r#"{"variables":{}}"#)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["errors"][0]["field"], "branch");
    assert!(api.feature.state.sink.journal.runs(Some(DEPLOY)).is_empty());
}

#[tokio::test]
async fn an_unknown_webhook_is_not_found() {
    let api = ready();
    let (status, _, _) = send(&api, run("no-such-webhook", "{}")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn running_while_the_webhooks_module_is_off_is_a_conflict() {
    let api = api(&format!("[modules]\nwebhooks = false\n\n{}", file()));
    let (status, _, _) = send(&api, run(DEPLOY, r#"{"variables":{"branch":"main"}}"#)).await;
    assert_eq!(status, StatusCode::CONFLICT);
}
