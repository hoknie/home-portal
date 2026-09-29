use axum::http::StatusCode;
use serde_json::json;

use super::automations::{api, get, send, write};
use super::workflows::{FILE, fields, ready};
use crate::features::AutomationsFeature;
use crate::features::tests::eventually;

#[tokio::test]
async fn running_answers_202_and_refuses_a_disabled_workflow_or_an_unknown_input() {
    let api = ready();
    let (status, _, body) = send(
        &api,
        write(
            "POST",
            "/api/workflows/revive/run",
            None,
            r#"{"inputs":{"service":"nas"}}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let run_id: u64 = body["run_id"].as_str().unwrap().parse().unwrap();
    let sink = api.feature.state.sink.clone();
    assert!(eventually(|| sink.journal.find(run_id).is_some()).await);
    let run = sink.journal.find(run_id).unwrap();
    assert_eq!(run.automation, "workflow:revive");
    assert_eq!(run.workflow.as_deref(), Some("revive"));
    assert_eq!(run.result.outcome, crate::types::Outcome::Succeeded);
    let (status, _, _) = send(&api, write("POST", "/api/workflows/spare/run", None, "")).await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, _, body) = send(
        &api,
        write(
            "POST",
            "/api/workflows/revive/run",
            None,
            r#"{"inputs":{"other":"x"}}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(fields(&body), vec!["inputs.other"]);
    let (status, _, _) = send(&api, write("POST", "/api/workflows/nope/run", None, "")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn running_while_the_module_is_off_is_a_conflict() {
    let api = api(&FILE.replace("workflows = true", "workflows = false"));
    let (status, _, body) = send(&api, write("POST", "/api/workflows/revive/run", None, "")).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body, "the workflows module is off");
    let (status, _, _) = send(&api, get(AutomationsFeature::WORKFLOWS)).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn the_catalogue_lists_kinds_operators_and_event_fields() {
    let api = ready();
    let (status, _, body) = send(&api, get(AutomationsFeature::WORKFLOW_CATALOGUE)).await;
    assert_eq!(status, StatusCode::OK);
    let http = body["kinds"]
        .as_array()
        .unwrap()
        .iter()
        .find(|kind| kind["name"] == "http")
        .unwrap();
    assert_eq!(http["group"], "actions");
    assert!(
        http["fields"]
            .as_array()
            .unwrap()
            .iter()
            .any(|field| field["name"] == "method" && field["type"] == "choice")
    );
    assert!(http["results"].as_array().unwrap().contains(&json!("json")));
    assert!(
        body["operators"]
            .as_array()
            .unwrap()
            .contains(&json!({"name": "is-empty", "takes_right": false}))
    );
    assert!(
        body["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["name"] == "manual")
    );
}
