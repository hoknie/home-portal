use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use serde_json::json;

use super::running::run;
use crate::types::{Ending, StepOutcome};

fn serve() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { continue };
            thread::spawn(move || answer(stream));
        }
    });
    port
}

fn answer(mut stream: std::net::TcpStream) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut head = String::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
            break;
        }
        head.push_str(&line);
    }
    let length = head
        .lines()
        .find_map(|line| {
            line.to_ascii_lowercase()
                .strip_prefix("content-length:")
                .map(|value| value.trim().parse::<usize>().unwrap_or(0))
        })
        .unwrap_or(0);
    let mut body = vec![0u8; length];
    let _ = reader.read_exact(&mut body);
    let path = head.split_whitespace().nth(1).unwrap_or("/").to_string();
    let reply = |status: &str, headers: &str, body: &[u8]| {
        let mut out = format!(
            "HTTP/1.1 {status}\r\ncontent-length: {}\r\nconnection: close\r\n{headers}\r\n",
            body.len()
        )
        .into_bytes();
        out.extend_from_slice(body);
        out
    };
    let bytes = if path == "/ok" {
        reply(
            "200 OK",
            "content-type: application/json\r\n",
            br#"{"state":"up","disks":[{"health":"ok"}]}"#,
        )
    } else if path == "/fail" {
        reply("503 Service Unavailable", "", b"down for maintenance")
    } else if let Some(left) = path.strip_prefix("/redirect/") {
        let left: usize = left.parse().unwrap_or(0);
        if left == 0 {
            reply("200 OK", "", b"arrived")
        } else {
            reply(
                "302 Found",
                &format!("location: /redirect/{}\r\n", left - 1),
                b"",
            )
        }
    } else if path == "/disks" {
        let disks: Vec<serde_json::Value> = (0..200)
            .map(|index| json!({"name": format!("sd{index}"), "health": "ok", "note": "x".repeat(150)}))
            .collect();
        let text = json!({"disks": disks}).to_string();
        reply(
            "200 OK",
            "content-type: application/json\r\n",
            text.as_bytes(),
        )
    } else if path == "/slow" {
        thread::sleep(Duration::from_secs(3));
        reply("200 OK", "", b"late")
    } else if path == "/big" {
        let size = 20 * 1024 * 1024;
        let mut out =
            format!("HTTP/1.1 200 OK\r\ncontent-length: {size}\r\nconnection: close\r\n\r\n")
                .into_bytes();
        out.extend(std::iter::repeat_n(b'x', size));
        out
    } else {
        let echoed = format!("{head}\n{}", String::from_utf8_lossy(&body));
        reply("200 OK", "", echoed.as_bytes())
    };
    let _ = stream.write_all(&bytes);
}

fn workflow(port: u16, step: &str) -> String {
    format!(
        "[[workflows]]\nid = \"w\"\ntitle = \"W\"\ninputs = [\"service\"]\ntimeout_seconds = 30\n[[workflows.steps]]\nid = \"call\"\nkind = \"http\"\n{}\n",
        step.replace("PORT", &port.to_string())
    )
}

#[tokio::test]
async fn an_answer_is_read_as_status_body_and_json() {
    let port = serve();
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
    let port = serve();
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
    let port = serve();
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
    let port = serve();
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
    let port = serve();
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
    let port = serve();
    let outcome = run(&workflow(port, "url = \"http://127.0.0.1:PORT/big\"")).await;
    assert_eq!(outcome.ending, Ending::Succeeded(None));
    let body = outcome.frame.steps["call"]["body"].as_str().unwrap().len();
    assert_eq!(body, 64 * 1024);
}

#[tokio::test]
async fn headers_and_a_json_body_are_sent() {
    let port = serve();
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
    let port = serve();
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
