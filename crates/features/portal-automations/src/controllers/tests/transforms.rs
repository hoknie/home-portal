use std::fs;

use axum::http::StatusCode;
use serde_json::json;

use super::automations::{api, get, revision, send, write};
use crate::features::AutomationsFeature;
use crate::features::tests::start;

const FILE: &str = r#"[modules]
workflows = true

[[workflows]]
id = "disks"
title = "Disks"

[[workflows.steps]]
id = "list"
kind = "http"
url = "http://nas.lan/disks"
"#;

#[tokio::test]
async fn a_transform_is_saved_as_a_list_of_operations_in_order_and_reads_back() {
    let api = api(FILE);
    start(&api.feature);
    let current = revision(&api).await;
    let steps = json!([
        {"id": "list", "kind": "http", "url": "http://nas.lan/disks"},
        {
            "id": "bad",
            "kind": "transform",
            "input": "{{steps.list.json.disks}}",
            "operations": [
                {"op": "filter", "where": {"left": "{{item.health}}", "op": "!=", "right": "ok"}},
                {"op": "pluck", "args": ["name"]},
                {"op": "join", "args": [", "]}
            ]
        }
    ]);
    let body = json!({"id": "disks", "title": "Disks", "steps": steps});
    let (status, _, answer) = send(
        &api,
        write(
            "PUT",
            "/api/workflows/disks",
            Some(&current),
            &body.to_string(),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{answer}");
    let saved = fs::read_to_string(api.folder.path().join("workflows/disks.toml")).unwrap();
    let filter = saved.find("op = \"filter\"").expect(&saved);
    let pluck = saved.find("op = \"pluck\"").expect(&saved);
    let join = saved.find("op = \"join\"").expect(&saved);
    assert!(filter < pluck && pluck < join, "{saved}");
    assert!(saved.contains("args = [\", \"]"), "{saved}");
    let (_, _, listed) = send(&api, get(AutomationsFeature::WORKFLOWS)).await;
    assert_eq!(listed["workflows"][0]["steps"], steps);
}

#[tokio::test]
async fn the_catalogue_lists_every_filter_and_operation_with_their_types() {
    let api = api(FILE);
    start(&api.feature);
    let (_, _, body) = send(&api, get(AutomationsFeature::WORKFLOW_CATALOGUE)).await;
    let join = body["filters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|filter| filter["name"] == "join")
        .unwrap();
    assert_eq!(
        join,
        &json!({"name": "join", "arguments": [{"name": "separator", "type": "text", "required": true, "choices": []}], "accepts": ["list"], "gives": "text", "element": false})
    );
    let names: Vec<&str> = body["operations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|operation| operation["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        vec!["filter", "map", "sort_by", "group_by", "count_by", "each"]
    );
}

async fn previewed(body: &str) -> (StatusCode, serde_json::Value) {
    let api = api(FILE);
    let (status, _, answer) = send(
        &api,
        write("POST", AutomationsFeature::TRANSFORM_PREVIEW, None, body),
    )
    .await;
    (status, answer)
}

#[tokio::test]
async fn previewing_a_chain_of_operations_answers_the_value_after_each() {
    let body = json!({
        "value": {"disks": [{"name": "sda", "health": "ok"}, {"name": "sdb", "health": "failing"}]},
        "filters": "",
        "operations": [
            {"op": "get", "args": ["disks"]},
            {"op": "filter", "where": {"left": "{{item.health}}", "op": "!=", "right": "ok"}},
            {"op": "pluck", "args": ["name"]}
        ]
    });
    let (status, answer) = previewed(&body.to_string()).await;
    assert_eq!(status, StatusCode::OK, "{answer}");
    assert_eq!(answer["steps"].as_array().unwrap().len(), 3);
    assert_eq!(
        answer["steps"][1]["value"],
        json!([{"name": "sdb", "health": "failing"}])
    );
    assert_eq!(answer["steps"][2]["value"], json!(["sdb"]));
}

#[tokio::test]
async fn a_failing_operation_stops_the_preview_and_the_input_filters_run_first() {
    let body = json!({
        "value": [" x ", "1"],
        "filters": "| first | trim",
        "operations": [{"op": "upper"}, {"op": "number"}, {"op": "lower"}]
    });
    let (status, answer) = previewed(&body.to_string()).await;
    assert_eq!(status, StatusCode::OK, "{answer}");
    assert_eq!(answer["input"]["value"], "x");
    assert_eq!(answer["steps"][0]["value"], "X");
    assert!(
        answer["steps"][1]["error"]
            .as_str()
            .unwrap()
            .contains("operation 2"),
        "{answer}"
    );
    assert_eq!(answer["steps"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn candidate_filters_get_their_example_from_the_value_after_the_input_filters() {
    let body = json!({
        "value": {"names": ["nas", "media"]},
        "filters": "| get(\"names\")",
        "examples": ["length", "join(\", \")", "nope"]
    });
    let (status, answer) = previewed(&body.to_string()).await;
    assert_eq!(status, StatusCode::OK, "{answer}");
    assert_eq!(answer["examples"][0]["value"], 2);
    assert_eq!(answer["examples"][1]["value"], "nas, media");
    assert!(answer["examples"][2]["error"].is_string(), "{answer}");
}

#[tokio::test]
async fn a_malformed_preview_is_refused_by_field_and_changes_nothing() {
    let (status, answer) = previewed("{\"filters\": 3}").await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{answer}");
    let (status, answer) =
        previewed(&json!({"value": 1, "operations": [{"op": "nope"}]}).to_string()).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{answer}");
    assert!(answer.to_string().contains("operations"), "{answer}");
}

#[tokio::test]
async fn names_the_editor_knows_are_read_by_the_operations() {
    let body = json!({
        "value": ["nas"],
        "operations": [{"op": "map", "to": "{{item}} of {{steps.list.json.owner}}"}],
        "names": {"steps.list.json.owner": "home"}
    });
    let (status, answer) = previewed(&body.to_string()).await;
    assert_eq!(status, StatusCode::OK, "{answer}");
    assert_eq!(answer["steps"][0]["value"], json!(["nas of home"]));
}
