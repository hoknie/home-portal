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
