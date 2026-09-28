use axum::http::StatusCode;
use serde_json::Value;

use super::automations::{Api, api, get, send, write};
use crate::features::tests::{eventually, start};

fn chain_file(parents: usize) -> String {
    let mut text = String::from("[modules]\nworkflows = true\n\n");
    for index in 0..parents {
        text.push_str(&format!(
            "[[workflows]]\nid = \"parent{index}\"\ntitle = \"Parent {index}\"\ninputs = [\"service\"]\n\n[[workflows.steps]]\nid = \"call\"\nkind = \"automation\"\nautomation = \"child{index}\"\nfields = {{ \"service.id\" = \"{{{{inputs.service}}}}\" }}\nwait = true\n\n[[automations]]\nid = \"start{index}\"\ntitle = \"Start {index}\"\nwhen = {{ event = \"manual\" }}\nworkflow = \"parent{index}\"\ninputs = {{ service = \"nas{index}\" }}\n\n[[automations]]\nid = \"child{index}\"\ntitle = \"Child {index}\"\nwhen = {{ event = \"service.status-changed\" }}\nrun = {{ script = \"restart.sh\", args = [\"{{{{service.id}}}}\"] }}\n\n"
        ));
    }
    text.push_str("[[workflows]]\nid = \"loop\"\ntitle = \"Loop\"\n\n[[workflows.steps]]\nid = \"again\"\nkind = \"automation\"\nautomation = \"looping\"\nwait = true\n\n[[automations]]\nid = \"looping\"\ntitle = \"Looping\"\nwhen = { event = \"manual\" }\nworkflow = \"loop\"\n");
    text
}

async fn run(api: &Api, automation: &str) -> u64 {
    let (status, _, body) = send(
        api,
        write(
            "POST",
            &format!("/api/automations/{automation}/run"),
            None,
            "",
        ),
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED, "{body}");
    body["run_id"].as_str().unwrap().parse().unwrap()
}

async fn finished(api: &Api, run_id: u64) -> Value {
    let sink = api.feature.state.sink.clone();
    assert!(
        eventually(|| sink.journal.find(run_id).is_some()).await,
        "run {run_id} did not finish"
    );
    send(api, get(&format!("/api/automations/runs/{run_id}")))
        .await
        .2
}

#[tokio::test]
async fn a_workflow_starts_an_automation_with_fields_and_waits_for_its_outcome() {
    let api = api(&chain_file(1));
    start(&api.feature);
    let parent = run(&api, "start0").await;
    let body = finished(&api, parent).await;
    assert_eq!(body["outcome"]["result"], "succeeded", "{body}");
    let entry = &body["trace"]["entries"][0];
    assert!(
        entry["detail"].as_str().unwrap().contains("child0 run"),
        "{entry}"
    );
    let child: u64 = entry["detail"]
        .as_str()
        .unwrap()
        .split_whitespace()
        .nth(2)
        .unwrap()
        .trim_end_matches(':')
        .parse()
        .unwrap();
    let child = finished(&api, child).await;
    assert_eq!(child["arguments"], serde_json::json!(["nas0"]), "{child}");
}

#[tokio::test]
async fn an_automation_cannot_start_the_run_it_belongs_to() {
    let api = api(&chain_file(1));
    start(&api.feature);
    let body = finished(&api, run(&api, "looping").await).await;
    assert_eq!(body["outcome"]["result"], "failed");
    assert!(
        body["outcome"]["reason"]
            .as_str()
            .unwrap()
            .contains("already started this run"),
        "{body}"
    );
}

#[tokio::test]
async fn parents_holding_every_dispatcher_slot_still_get_their_children_run() {
    let api = api(&chain_file(4));
    start(&api.feature);
    let mut parents = Vec::new();
    for index in 0..4 {
        parents.push(run(&api, &format!("start{index}")).await);
    }
    for parent in parents {
        assert_eq!(
            finished(&api, parent).await["outcome"]["result"],
            "succeeded"
        );
    }
}
