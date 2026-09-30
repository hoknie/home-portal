use std::fs;

use axum::http::StatusCode;

use super::setup::{CAMERA, ready};
use crate::controllers::tests::automations::{get, revision, send, write};
use crate::features::AutomationsFeature;

fn motion(title: &str) -> String {
    format!(
        r#"{{"id":"watch","title":"{title}","when":{{"event":"webhook.received","webhooks":["{CAMERA}"]}},"workflow":"alarm","inputs":{{"camera":"{{{{webhook.camera}}}}"}}}}"#
    )
}

#[tokio::test]
async fn a_webhook_automation_that_runs_a_workflow_is_saved_and_saved_again() {
    let api = ready();
    let current = revision(&api).await;
    let (status, created, _) = send(
        &api,
        write(
            "POST",
            AutomationsFeature::COLLECTION,
            Some(&current),
            &motion("Motion"),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let uri = format!("{}/watch", AutomationsFeature::COLLECTION);
    let (status, _, _) = send(
        &api,
        write("PUT", &uri, created.as_deref(), &motion("Motion now")),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let text = fs::read_to_string(api.folder.path().join("automations.toml")).unwrap();
    assert!(text.contains("title = \"Motion now\""));
    assert!(text.contains("workflow = \"alarm\""));
    assert!(text.contains("{{webhook.camera}}"));
    assert!(!text.contains("run"));
}

#[tokio::test]
async fn a_workflow_automation_is_listed_with_no_run() {
    let api = ready();
    let current = revision(&api).await;
    send(
        &api,
        write(
            "POST",
            AutomationsFeature::COLLECTION,
            Some(&current),
            &motion("Motion"),
        ),
    )
    .await;
    let (_, _, body) = send(&api, get(AutomationsFeature::COLLECTION)).await;
    let motion = body["automations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|automation| automation["id"] == "motion")
        .unwrap();
    assert_eq!(motion["run"], serde_json::Value::Null);
    assert_eq!(motion["workflow"]["id"], "alarm");
    assert_eq!(motion["workflow"]["inputs"]["camera"], "{{webhook.camera}}");
}

#[tokio::test]
async fn an_error_of_the_entry_is_named_without_the_section() {
    let api = ready();
    let current = revision(&api).await;
    let odd = r#"{"id":"odd","title":"Odd","when":{"event":"manual"},"workflow":"alarm","inputs":{"x":"1"}}"#;
    let (status, _, body) = send(
        &api,
        write("POST", AutomationsFeature::COLLECTION, Some(&current), odd),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let fields: Vec<&str> = body["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|error| error["field"].as_str().unwrap())
        .collect();
    assert_eq!(fields, vec!["inputs.x"]);
}

#[tokio::test]
async fn webhook_choices_carry_their_action() {
    let api = ready();
    let (_, _, body) = send(&api, get("/api/automations/catalogue")).await;
    let actions: std::collections::BTreeMap<String, String> = body["choices"]["webhooks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|choice| {
            (
                choice["name"].as_str().unwrap().to_string(),
                choice["action"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    assert_eq!(actions["Camera"], "event");
    assert_eq!(actions["Deploy"], "script");
    assert_eq!(actions["Alarm"], "workflow");
}

#[tokio::test]
async fn a_workflow_lists_the_webhook_variables_that_reach_it() {
    let api = ready();
    let (_, _, body) = send(&api, get("/api/workflows")).await;
    let used_by = |id: &str| {
        body["workflows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|workflow| workflow["id"] == id)
            .unwrap()["used_by"]
            .clone()
    };
    let alarm = used_by("alarm");
    let motion = alarm
        .as_array()
        .unwrap()
        .iter()
        .find(|usage| usage["id"] == "motion")
        .unwrap();
    assert_eq!(motion["variables"], serde_json::json!(["camera"]));
    assert_eq!(used_by("sizes")[0]["kind"], "webhook");
    assert_eq!(used_by("sizes")[0]["variables"], serde_json::json!([]));
}
