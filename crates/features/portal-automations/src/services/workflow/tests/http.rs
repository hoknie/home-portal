use std::time::Duration;

use portal_testing::{Answer, FakeHttp, Request};
use serde_json::json;

use super::running::run;
use crate::types::{Ending, StepOutcome};

pub async fn serve() -> u16 {
    FakeHttp::start(reply).await.address.port()
}

fn reply(request: &Request) -> Answer {
    let path = request.path.as_str();
    if path == "/ok" {
        Answer::json(r#"{"state":"up","disks":[{"health":"ok"}]}"#)
    } else if path == "/metadata" {
        Answer::status(302).with_header("location", "http://169.254.169.254/latest/meta-data")
    } else if path == "/fail" {
        Answer::status(503).with_body("down for maintenance")
    } else if let Some(left) = path.strip_prefix("/redirect/") {
        match left.parse::<usize>().unwrap_or(0) {
            0 => Answer::status(200).with_body("arrived"),
            left => Answer::status(302).with_header("location", &format!("/redirect/{}", left - 1)),
        }
    } else if path == "/disks" {
        let disks: Vec<serde_json::Value> = (0..200)
            .map(|index| json!({"name": format!("sd{index}"), "health": "ok", "note": "x".repeat(150)}))
            .collect();
        Answer::json(&json!({"disks": disks}).to_string())
    } else if path == "/slow" {
        Answer::status(200)
            .with_body("late")
            .after(Duration::from_secs(3))
    } else if path == "/big" {
        Answer::status(200).with_body(vec![b'x'; 20 * 1024 * 1024])
    } else {
        Answer::status(200).with_body(format!("{}\r\n\n{}", request.head, request.body))
    }
}

fn workflow(port: u16, step: &str) -> String {
    format!(
        "[[workflows]]\nid = \"w\"\ntitle = \"W\"\ninputs = [\"service\"]\ntimeout_seconds = 30\n[[workflows.steps]]\nid = \"call\"\nkind = \"http\"\n{}\n",
        step.replace("PORT", &port.to_string())
    )
}

#[tokio::test]
async fn an_answer_is_read_as_status_body_and_json() {
    let port = serve().await;
    let outcome = run(&workflow(port, "url = \"http://127.0.0.1:PORT/ok\"")).await;
    assert_eq!(outcome.ending, Ending::Succeeded(None));
    let result = &outcome.frame.steps["call"];
    assert_eq!(result["status"], json!(200));
    assert_eq!(result["json"]["disks"][0]["health"], json!("ok"));
    assert_eq!(result["headers"]["content-type"], json!("application/json"));
    assert!(outcome.trace.entries[0].detail.ends_with("→ 200"));
}

#[tokio::test]
async fn a_failing_request_fails_the_step_with_its_status() {
    let port = serve().await;
    let outcome = run(&workflow(port, "url = \"http://127.0.0.1:PORT/fail\"")).await;
    let Ending::Failed(reason) = &outcome.ending else {
        panic!("{:?}", outcome.ending);
    };
    assert!(reason.ends_with("→ 503"), "{reason}");
    assert_eq!(outcome.trace.entries[0].outcome, StepOutcome::Failed);
    assert_eq!(
        outcome.trace.entries[0].output.as_deref(),
        Some("down for maintenance")
    );
}

#[tokio::test]
async fn a_tolerated_error_keeps_going_with_the_status() {
    let port = serve().await;
    let outcome = run(&workflow(
        port,
        "url = \"http://127.0.0.1:PORT/fail\"\nfail_on_error = false",
    ))
    .await;
    assert_eq!(outcome.ending, Ending::Succeeded(None));
    assert_eq!(outcome.frame.steps["call"]["status"], json!(503));
}

#[tokio::test]
async fn more_than_five_redirects_fail_and_five_are_followed() {
    let port = serve().await;
    let followed = run(&workflow(
        port,
        "url = \"http://127.0.0.1:PORT/redirect/5\"",
    ))
    .await;
    assert_eq!(followed.frame.steps["call"]["body"], json!("arrived"));
    let refused = run(&workflow(
        port,
        "url = \"http://127.0.0.1:PORT/redirect/6\"",
    ))
    .await;
    let Ending::Failed(reason) = &refused.ending else {
        panic!("{:?}", refused.ending);
    };
    assert!(reason.contains("redirects"), "{reason}");
}

#[tokio::test]
async fn a_slow_server_hits_the_step_timeout() {
    let port = serve().await;
    let outcome = run(&workflow(
        port,
        "url = \"http://127.0.0.1:PORT/slow\"\ntimeout_seconds = 1",
    ))
    .await;
    let Ending::Failed(reason) = &outcome.ending else {
        panic!("{:?}", outcome.ending);
    };
    assert!(reason.contains("timed out"), "{reason}");
}

#[tokio::test]
async fn a_huge_answer_is_capped() {
    let port = serve().await;
    let outcome = run(&workflow(port, "url = \"http://127.0.0.1:PORT/big\"")).await;
    assert_eq!(outcome.ending, Ending::Succeeded(None));
    let body = outcome.frame.steps["call"]["body"].as_str().unwrap().len();
    assert_eq!(body, 64 * 1024);
}

#[tokio::test]
async fn headers_and_a_json_body_are_sent() {
    let port = serve().await;
    let outcome = run(&workflow(
        port,
        "method = \"POST\"\nurl = \"http://127.0.0.1:PORT/echo\"\nheaders = { X-Service = \"{{inputs.service}}\" }\nbody = '{\"service\": \"{{inputs.service}}\"}'",
    ))
    .await;
    let echoed = outcome.frame.steps["call"]["body"]
        .as_str()
        .unwrap()
        .to_ascii_lowercase();
    assert!(echoed.starts_with("post /echo"), "{echoed}");
    assert!(echoed.contains("x-service: nas"), "{echoed}");
    assert!(
        echoed.contains("content-type: application/json"),
        "{echoed}"
    );
    assert!(echoed.contains("{\"service\": \"nas\"}"), "{echoed}");
}

#[tokio::test]
async fn a_long_answer_keeps_its_shape_while_its_body_is_cut() {
    let port = serve().await;
    let outcome = run(&workflow(port, "url = \"http://127.0.0.1:PORT/disks\"")).await;
    assert_eq!(outcome.ending, Ending::Succeeded(None));
    let entry = &outcome.trace.entries[0];
    assert_eq!(entry.output.as_deref().map(str::len), Some(4 * 1024));
    let shape: serde_json::Value = serde_json::from_str(entry.shape.as_deref().unwrap()).unwrap();
    let disks = shape["disks"].as_array().unwrap();
    assert_eq!(disks.len(), 3);
    assert_eq!(disks[0]["name"], json!("sd0"));
    assert_eq!(disks[0]["note"].as_str().unwrap().len(), 150);
}

fn request(url: &str) -> crate::types::HttpRequest {
    crate::types::HttpRequest {
        method: "POST".to_string(),
        url: url.to_string(),
        headers: Vec::new(),
        body: None,
        timeout: Duration::from_secs(2),
    }
}

#[tokio::test]
async fn a_request_to_the_portal_host_is_refused_before_it_is_sent() {
    let client = crate::clients::HttpClient::new().unwrap();
    let refused = client
        .send(&request("http://127.0.0.1:2019/load"))
        .await
        .unwrap_err();
    assert!(
        refused.contains("127.0.0.1 is a loopback address"),
        "{refused}"
    );
    let named = client
        .send(&request("http://localhost:2019/load"))
        .await
        .unwrap_err();
    assert!(named.contains("loopback"), "{named}");
    let link_local = client
        .send(&request("http://[fe80::1]/"))
        .await
        .unwrap_err();
    assert!(link_local.contains("link-local"), "{link_local}");
}

#[tokio::test]
async fn a_redirect_into_the_metadata_address_is_not_followed() {
    let port = serve().await;
    let outcome = run(&workflow(port, "url = \"http://127.0.0.1:PORT/metadata\"")).await;
    let Ending::Failed(reason) = &outcome.ending else {
        panic!("{:?}", outcome.ending);
    };
    assert!(
        reason.contains("169.254.169.254 is a link-local address"),
        "{reason}"
    );
}
