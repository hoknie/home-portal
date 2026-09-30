use super::setup::{ALARM, CAMERA, ECHO, INPUT, call, finished, last_of, ready};
use crate::controllers::tests::automations::{send, write};

#[tokio::test]
async fn a_nested_value_from_the_body_is_an_argument() {
    let api = ready();
    let run_id = call(
        &api,
        ECHO,
        "application/json",
        r#"{"repository":{"name":"portal"}}"#,
    )
    .await
    .unwrap();
    let body = finished(&api, run_id).await;
    assert_eq!(body["outcome"]["result"], "succeeded");
    assert_eq!(
        body["outcome"]["stdout"]["tail"],
        "portal {\"repository\":{\"name\":\"portal\"}}\n"
    );
}

#[tokio::test]
async fn a_body_that_is_not_json_is_its_text() {
    let api = ready();
    let run_id = call(&api, ECHO, "text/plain", "hello").await.unwrap();
    let body = finished(&api, run_id).await;
    assert_eq!(body["arguments"], serde_json::json!(["", "hello"]));
}

#[tokio::test]
async fn the_body_is_on_standard_input_and_not_in_the_environment() {
    let api = ready();
    let run_id = call(
        &api,
        INPUT,
        "application/json",
        r#"{"ref":"main","commits":2}"#,
    )
    .await
    .unwrap();
    let body = finished(&api, run_id).await;
    let output = body["outcome"]["stdout"]["tail"].as_str().unwrap();
    let input: serde_json::Value = serde_json::from_str(output.lines().next().unwrap()).unwrap();
    assert_eq!(
        input["webhook.body"],
        serde_json::json!({"ref": "main", "commits": 2})
    );
    assert!(output.contains("PORTAL_WEBHOOK_ID="));
    assert!(!output.contains("PORTAL_WEBHOOK_BODY"));
}

#[tokio::test]
async fn a_body_larger_than_a_field_reaches_an_object_input_whole() {
    let api = ready();
    let large = "x".repeat(20 * 1024);
    let payload = format!(r#"{{"text":"{large}","tail":"end"}}"#);
    let run_id = call(&api, ALARM, "application/json", &payload)
        .await
        .unwrap();
    let body = finished(&api, run_id).await;
    assert_eq!(body["outcome"]["result"], "succeeded", "{body}");
    assert_eq!(body["outcome"]["reason"], "end end");
}

#[tokio::test]
async fn an_event_webhook_passes_its_data_to_a_workflow_through_an_automation() {
    let api = ready();
    call(
        &api,
        CAMERA,
        "application/json",
        r#"{"camera":"gate","zones":[1,3]}"#,
    )
    .await;
    let body = last_of(&api, "motion").await;
    assert_eq!(body["outcome"]["result"], "succeeded", "{body}");
    assert_eq!(body["outcome"]["reason"], "gate [1,3]");
}

#[tokio::test]
async fn running_a_webhook_from_the_interface_takes_a_body() {
    let api = ready();
    let uri = format!("/api/webhooks/{ALARM}/run");
    let (status, _, answer) = send(
        &api,
        write(
            "POST",
            &uri,
            None,
            r#"{"variables":{},"body":{"tail":"by hand"}}"#,
        ),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::ACCEPTED, "{answer}");
    let run_id: u64 = answer["run_id"].as_str().unwrap().parse().unwrap();
    let body = finished(&api, run_id).await;
    assert_eq!(body["outcome"]["reason"], "by hand by hand");
}

#[tokio::test]
async fn run_now_fills_the_variables_the_workflow_reads() {
    let api = ready();
    let (status, _, answer) =
        send(&api, write("POST", "/api/automations/peek/run", None, "")).await;
    assert_eq!(status, axum::http::StatusCode::ACCEPTED, "{answer}");
    let run_id: u64 = answer["run_id"].as_str().unwrap().parse().unwrap();
    let body = finished(&api, run_id).await;
    assert_eq!(body["outcome"]["reason"], "camera camera");
}
