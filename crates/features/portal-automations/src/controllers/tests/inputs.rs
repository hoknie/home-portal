use std::fs;

use axum::http::StatusCode;
use serde_json::json;

use super::automations::{api, get, revision, send, write};
use crate::features::AutomationsFeature;
use crate::features::tests::start;

const FILE: &str = r#"[modules]
workflows = true

[[workflows]]
id = "check-hosts"
title = "Check hosts"
inputs = ["service", { name = "hosts", type = "list", default = ["nas"] }]

[[workflows.steps]]
id = "n"
kind = "nothing"

[[automations]]
id = "morning"
title = "Morning"
when = { event = "manual" }
workflow = "check-hosts"
inputs = { hosts = ["nas", "router"] }
"#;

#[tokio::test]
async fn typed_inputs_are_listed_with_their_types_and_a_literal_list_reads_back() {
    let api = api(FILE);
    start(&api.feature);
    let (status, _, body) = send(&api, get(AutomationsFeature::WORKFLOWS)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body["workflows"][0]["inputs"],
        json!([
            {"name": "service", "type": "text", "default": null, "description": null},
            {"name": "hosts", "type": "list", "default": ["nas"], "description": null}
        ])
    );
    let (_, _, automations) = send(&api, get(AutomationsFeature::COLLECTION)).await;
    assert_eq!(
        automations["automations"][0]["workflow"]["inputs"]["hosts"],
        json!(["nas", "router"])
    );
}

#[tokio::test]
async fn saving_keeps_plain_names_and_writes_typed_inputs_as_tables() {
    let api = api(FILE);
    start(&api.feature);
    let current = revision(&api).await;
    let body = json!({
        "id": "check-hosts",
        "title": "Check hosts",
        "inputs": ["service", {"name": "hosts", "type": "list", "default": ["nas", "nvr"], "description": "Hosts to check"}],
        "steps": [{"id": "n", "kind": "nothing"}]
    });
    let (status, _, answer) = send(
        &api,
        write(
            "PUT",
            "/api/workflows/check-hosts",
            Some(&current),
            &body.to_string(),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{answer}");
    let saved = fs::read_to_string(api.folder.path().join("home-portal.toml")).unwrap();
    assert!(
        saved.contains(r#"inputs = ["service", { name = "hosts", type = "list", default = ["nas", "nvr"], description = "Hosts to check" }]"#),
        "{saved}"
    );
}

#[tokio::test]
async fn running_with_a_value_that_is_not_of_its_type_is_refused_by_field() {
    let api = api(FILE);
    start(&api.feature);
    let (status, _, body) = send(
        &api,
        write(
            "POST",
            "/api/workflows/check-hosts/run",
            None,
            r#"{"inputs": {"hosts": "not a list"}}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["errors"][0]["field"], "inputs.hosts");
    let (status, _, _) = send(
        &api,
        write(
            "POST",
            "/api/workflows/check-hosts/run",
            None,
            r#"{"inputs": {"hosts": ["nas", "router"], "service": "nas"}}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
}

#[tokio::test]
async fn a_literal_of_the_wrong_type_in_an_automation_is_refused() {
    let api = api(FILE);
    let current = revision(&api).await;
    let (status, _, body) = send(
        &api,
        write(
            "PUT",
            "/api/automations/morning",
            Some(&current),
            &json!({"id": "morning", "title": "Morning", "when": {"event": "manual"}, "workflow": "check-hosts", "inputs": {"hosts": 3}}).to_string(),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert!(body.to_string().contains("inputs.hosts"), "{body}");
}

#[tokio::test]
async fn the_portal_values_are_answered_as_templates_read_them() {
    let api = api(FILE);
    let (status, _, body) = send(&api, get(AutomationsFeature::WORKFLOW_PORTAL)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["services"][0]["name"], "Media");
    assert_eq!(body["network"]["port"], 8080);
    assert_eq!(body["modules"]["workflows"]["is_enabled"], true);
    assert_eq!(body["modules"]["users"]["is_enabled"], false);
}
