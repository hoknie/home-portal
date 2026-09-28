use axum::http::StatusCode;
use serde_json::Value;

use super::automations::{Api, api, get, revision, send, write};
use crate::features::AutomationsFeature;
use crate::features::tests::{eventually, start};

const HOOK: &str = "3c1e2d4f-5a6b-4c7d-8e9f-0a1b2c3d4e5f";

fn file(modules: &str) -> String {
    format!(
        r#"[modules]
{modules}

[[workflows]]
id = "greet"
title = "Greet"
inputs = ["who"]

[[workflows.steps]]
id = "greeting"
kind = "set"
variable = "greeting"
value = "hello {{{{inputs.who}}}}"

[[workflows.steps]]
id = "done"
kind = "stop"
outcome = "succeeded"
reason = "{{{{vars.greeting}}}}"

[[workflows]]
id = "slow"
title = "Slow"

[[workflows.steps]]
id = "first"
kind = "set"
variable = "phase"
value = "waiting"

[[workflows.steps]]
id = "pause"
kind = "wait"
seconds = 600

[[automations]]
id = "hello"
title = "Hello"
cooldown_seconds = 300
when = {{ event = "manual" }}
workflow = "greet"
inputs = {{ who = "world" }}

[[automations]]
id = "sleepy"
title = "Sleepy"
when = {{ event = "manual" }}
workflow = "slow"

[[webhooks]]
id = "{HOOK}"
title = "Greeter"
variables = ["name"]
action = "script"
workflow = "greet"
inputs = {{ who = "{{{{webhook.name}}}}" }}
"#
    )
}

fn ready(modules: &str) -> Api {
    let api = api(&file(modules));
    start(&api.feature);
    api
}

async fn run(api: &Api, automation: &str) -> u64 {
    let uri = format!("/api/automations/{automation}/run");
    let (status, _, body) = send(api, write("POST", &uri, None, "")).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    body["run_id"].as_str().unwrap().parse().unwrap()
}

async fn finished(api: &Api, run_id: u64) -> Value {
    let sink = api.feature.state.sink.clone();
    assert!(eventually(|| sink.journal.find(run_id).is_some()).await);
    send(api, get(&format!("/api/automations/runs/{run_id}")))
        .await
        .2
}

#[tokio::test]
async fn an_automation_runs_its_workflow_with_rendered_inputs_and_a_trace() {
    let api = ready("workflows = true");
    let run_id = run(&api, "hello").await;
    let body = finished(&api, run_id).await;
    assert_eq!(body["outcome"]["result"], "succeeded");
    assert_eq!(body["outcome"]["reason"], "hello world");
    assert_eq!(body["workflow"], "greet");
    let steps: Vec<&str> = body["trace"]["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["step"].as_str().unwrap())
        .collect();
    assert_eq!(steps, vec!["greeting", "done"]);
}

#[tokio::test]
async fn an_automation_with_both_a_script_and_a_workflow_is_refused() {
    let api = ready("workflows = true");
    let current = revision(&api).await;
    let both = r#"{"id":"both","title":"Both","when":{"event":"manual"},"run":{"script":"restart.sh"},"workflow":"greet","inputs":{"who":"x"}}"#;
    let (status, _, body) = send(
        &api,
        write("POST", AutomationsFeature::COLLECTION, Some(&current), both),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body.to_string().contains("workflow"));
    let unknown = r#"{"id":"lost","title":"Lost","when":{"event":"manual"},"workflow":"nope"}"#;
    let (status, _, _) = send(
        &api,
        write(
            "POST",
            AutomationsFeature::COLLECTION,
            Some(&current),
            unknown,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let undeclared = r#"{"id":"odd","title":"Odd","when":{"event":"manual"},"workflow":"greet","inputs":{"whom":"x"}}"#;
    let (status, _, _) = send(
        &api,
        write(
            "POST",
            AutomationsFeature::COLLECTION,
            Some(&current),
            undeclared,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn a_webhook_runs_its_workflow_with_the_variable_as_input() {
    let api = ready("workflows = true");
    let request = axum::http::Request::post(format!("/webhook/{HOOK}"))
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .body(axum::body::Body::from(r#"{"name":"Ann"}"#))
        .unwrap();
    let (status, _, body) = send(&api, request).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let run_id: u64 = body["run_id"].as_str().unwrap().parse().unwrap();
    let body = finished(&api, run_id).await;
    assert_eq!(body["outcome"]["result"], "succeeded");
    assert_eq!(body["outcome"]["reason"], "hello Ann");
    assert_eq!(body["workflow"], "greet");
}

#[tokio::test]
async fn a_trigger_while_the_module_is_off_is_recorded_as_refused() {
    let api = ready("workflows = false");
    let run_id = run(&api, "hello").await;
    let body = finished(&api, run_id).await;
    assert_eq!(body["outcome"]["result"], "refused");
    assert!(
        body["outcome"]["reason"]
            .as_str()
            .unwrap()
            .starts_with("workflows-off")
    );
}

#[tokio::test]
async fn the_cooldown_applies_to_workflow_runs() {
    let api = ready("workflows = true");
    let first = run(&api, "hello").await;
    finished(&api, first).await;
    let (status, _, _) = send(&api, write("POST", "/api/automations/hello/run", None, "")).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn a_running_workflow_shows_its_trace_so_far_and_stops_at_once() {
    let api = ready("workflows = true");
    let run_id = run(&api, "sleepy").await;
    let sink = api.feature.state.sink.clone();
    assert!(
        eventually(|| sink.active.trace(run_id).is_some_and(|trace| trace
            .lock()
            .unwrap()
            .entries
            .len()
            == 2))
        .await
    );
    let (_, _, body) = send(&api, get(&format!("/api/automations/runs/{run_id}"))).await;
    assert_eq!(body["outcome"]["result"], "running");
    assert_eq!(body["workflow"], "slow");
    assert_eq!(body["trace"]["entries"][0]["outcome"], "succeeded");
    assert_eq!(body["trace"]["entries"][1]["step"], "pause");
    assert_eq!(body["trace"]["entries"][1]["outcome"], "running");
    let stop = format!("/api/automations/runs/{run_id}/stop");
    let (status, _, _) = send(&api, write("POST", &stop, None, "")).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let body = finished(&api, run_id).await;
    assert_eq!(body["outcome"]["result"], "stopped");
    assert_eq!(body["trace"]["entries"][1]["outcome"], "stopped");
}

#[tokio::test]
async fn the_runs_of_one_workflow_are_filtered_by_its_id() {
    let api = ready("workflows = true");
    let from_automation = run(&api, "hello").await;
    finished(&api, from_automation).await;
    let request = axum::http::Request::post(format!("/webhook/{HOOK}"))
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .body(axum::body::Body::from(r#"{"name":"Ann"}"#))
        .unwrap();
    let (_, _, body) = send(&api, request).await;
    let from_webhook: u64 = body["run_id"].as_str().unwrap().parse().unwrap();
    finished(&api, from_webhook).await;
    let other = run(&api, "sleepy").await;
    let (_, _, listed) = send(&api, get("/api/automations/runs?workflow=greet")).await;
    let ids: Vec<String> = listed["runs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["id"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        ids,
        vec![from_webhook.to_string(), from_automation.to_string()]
    );
    let (_, _, slow) = send(&api, get("/api/automations/runs?workflow=slow")).await;
    assert_eq!(slow["runs"][0]["id"], other.to_string());
    api.feature.state.sink.stop(other, "admin");
}
