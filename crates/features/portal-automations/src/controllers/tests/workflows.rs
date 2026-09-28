use std::fs;

use axum::http::StatusCode;
use serde_json::{Value, json};

use super::automations::{Api, api, get, send, write};
use crate::features::AutomationsFeature;
use crate::features::tests::{eventually, start};

const FILE: &str = r#"[modules]
workflows = true

# Brings a service back.
[[workflows]]
id = "revive"
title = "Revive" # shown in the list
inputs = ["service"]

[[workflows.steps]]
id = "first"
kind = "probe"
service = "{{inputs.service}}"

[[workflows]]
id = "spare"
title = "Spare"
enabled = false

[[workflows.steps]]
id = "note"
kind = "set"
variable = "note"
value = "x"

[[automations]]
id = "nas-down"
title = "NAS down"
when = { event = "manual" }
workflow = "revive"
inputs = { service = "nas" }
"#;

fn ready() -> Api {
    let api = api(FILE);
    start(&api.feature);
    api
}

async fn revision(api: &Api) -> String {
    send(api, get(AutomationsFeature::WORKFLOWS))
        .await
        .1
        .unwrap()
}

fn text(api: &Api) -> String {
    fs::read_to_string(api.folder.path().join("home-portal.toml")).unwrap()
}

fn fields(body: &Value) -> Vec<String> {
    body["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|error| error["field"].as_str().unwrap().to_string())
        .collect()
}

fn tree() -> Value {
    json!({
        "id": "revive",
        "title": "Revive",
        "inputs": ["service"],
        "timeout_seconds": 600,
        "steps": [
            {"id": "first", "kind": "probe", "service": "{{inputs.service}}"},
            {
                "id": "down",
                "label": "Is it down?",
                "kind": "if",
                "condition": {"left": "{{steps.first.state}}", "op": "!=", "right": "up"},
                "then": [
                    {
                        "id": "retry",
                        "kind": "loop",
                        "repeat": 3,
                        "body": [
                            {"id": "restart", "kind": "http", "method": "POST", "url": "http://host/restart", "body": "{\"now\":true}"},
                            {"id": "settle", "kind": "wait", "seconds": 5}
                        ]
                    }
                ],
                "else": [{"id": "fine", "kind": "stop", "outcome": "succeeded", "reason": "already up"}]
            },
            {
                "id": "both",
                "kind": "parallel",
                "branches": [
                    [{"id": "a", "kind": "status", "service": "nas"}],
                    [{"id": "b", "kind": "status", "service": "router"}]
                ]
            }
        ]
    })
}

#[tokio::test]
async fn the_list_carries_each_workflow_with_its_users_and_the_revision() {
    let api = ready();
    let (status, etag, body) = send(&api, get(AutomationsFeature::WORKFLOWS)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(etag.is_some());
    let revive = &body["workflows"][0];
    assert_eq!(revive["id"], "revive");
    assert_eq!(revive["enabled"], true);
    assert_eq!(revive["timeout_seconds"], 300);
    assert_eq!(revive["steps"][0]["kind"], "probe");
    assert_eq!(
        revive["used_by"],
        json!([{"kind": "automation", "id": "nas-down", "title": "NAS down"}])
    );
    assert_eq!(revive["last_run"], Value::Null);
    assert_eq!(body["workflows"][1]["enabled"], false);
}

#[tokio::test]
async fn saving_a_tree_keeps_the_comment_above_the_entry_and_reads_back() {
    let api = ready();
    let current = revision(&api).await;
    let (status, etag, body) = send(
        &api,
        write(
            "PUT",
            "/api/workflows/revive",
            Some(&current),
            &tree().to_string(),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_ne!(etag.unwrap(), current);
    let saved = text(&api);
    assert!(saved.contains("# Brings a service back.\n[[workflows]]"));
    assert!(saved.contains("title = \"Revive\" # shown in the list"));
    assert!(saved.contains("[[workflows.steps.then]]"));
    assert!(saved.contains("[[workflows.steps.then.body]]"));
    assert!(saved.contains("[[workflows.steps.else]]"));
    assert!(saved.contains("timeout_seconds = 600"));
    let (_, _, listed) = send(&api, get(AutomationsFeature::WORKFLOWS)).await;
    let steps = &listed["workflows"][0]["steps"];
    assert_eq!(steps, &tree()["steps"]);
}

#[tokio::test]
async fn a_nested_error_is_answered_with_its_full_path() {
    let api = ready();
    let current = revision(&api).await;
    let mut wrong = tree();
    wrong["steps"][1]["then"][0]["body"][0]["url"] = json!("ftp://x");
    wrong["steps"][1]["else"][0]["reason"] = json!("{{steps.both.x}}");
    let (status, _, body) = send(
        &api,
        write(
            "PUT",
            "/api/workflows/revive",
            Some(&current),
            &wrong.to_string(),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(fields(&body), vec!["steps[1].then[0].body[0].url"]);
    wrong["steps"][1]["then"][0]["body"][0]["url"] = json!("http://host/restart");
    let (status, _, body) = send(
        &api,
        write(
            "PUT",
            "/api/workflows/revive",
            Some(&current),
            &wrong.to_string(),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(fields(&body), vec!["steps[1].else[0].reason"]);
    assert_eq!(text(&api), FILE);
}

#[tokio::test]
async fn a_new_workflow_is_appended_and_a_taken_id_is_refused() {
    let api = ready();
    let current = revision(&api).await;
    let mut new = tree();
    new["id"] = json!("heal");
    let (status, etag, body) = send(
        &api,
        write(
            "POST",
            AutomationsFeature::WORKFLOWS,
            Some(&current),
            &new.to_string(),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["id"], "heal");
    assert_eq!(body["used_by"], json!([]));
    let (status, _, body) = send(
        &api,
        write(
            "POST",
            AutomationsFeature::WORKFLOWS,
            Some(&etag.unwrap()),
            &new.to_string(),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(fields(&body), vec!["id"]);
    let (status, _, _) = send(
        &api,
        write(
            "POST",
            AutomationsFeature::WORKFLOWS,
            None,
            &new.to_string(),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::PRECONDITION_REQUIRED);
}

#[tokio::test]
async fn deleting_a_workflow_in_use_is_refused_naming_its_users() {
    let api = ready();
    let current = revision(&api).await;
    let (status, _, body) = send(
        &api,
        write("DELETE", "/api/workflows/revive", Some(&current), ""),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(body.as_str().unwrap().contains("nas-down"));
    assert_eq!(text(&api), FILE);
    let (status, _, body) = send(
        &api,
        write("DELETE", "/api/workflows/spare", Some(&current), ""),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["workflows"].as_array().unwrap().len(), 1);
    assert!(!text(&api).contains("spare"));
}

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

#[tokio::test]
async fn workflow_tags_join_the_tags_shared_with_automations_and_webhooks() {
    let api = api(&FILE.replace(
        "inputs = [\"service\"]",
        "inputs = [\"service\"]\ntags = [\"media\"]",
    ));
    let (_, _, body) = send(&api, get(AutomationsFeature::CATALOGUE)).await;
    assert!(
        body["choices"]["tags"]
            .as_array()
            .unwrap()
            .contains(&json!("media"))
    );
}

#[tokio::test]
async fn an_old_telegram_step_loads_and_is_saved_as_a_notify_step_to_telegram() {
    let api = ready();
    let current = revision(&api).await;
    let body = json!({
        "id": "revive",
        "title": "Revive",
        "inputs": ["service"],
        "steps": [
            {"id": "tell", "kind": "telegram", "text": "{{inputs.service}} is down"},
            {"id": "both", "kind": "parallel", "branches": [[{"id": "a", "kind": "telegram", "text": "a"}], [{"id": "b", "kind": "wait", "seconds": 1}]]}
        ]
    });
    let (status, _, answer) = send(
        &api,
        write(
            "PUT",
            "/api/workflows/revive",
            Some(&current),
            &body.to_string(),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{answer}");
    let saved = text(&api);
    assert!(!saved.contains("kind = \"telegram\""), "{saved}");
    assert_eq!(
        saved.matches("channel = \"telegram\"").count(),
        2,
        "{saved}"
    );
    assert_eq!(answer["steps"][0]["kind"], "notify");
    assert_eq!(answer["steps"][0]["channel"], "telegram");
}
