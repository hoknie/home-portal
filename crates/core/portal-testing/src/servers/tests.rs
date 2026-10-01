use std::time::{Duration, Instant};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use super::FakeHttp;
use crate::types::Answer;

async fn send(server: &FakeHttp, request: &str) -> String {
    let mut socket = TcpStream::connect(server.address).await.unwrap();
    socket.write_all(request.as_bytes()).await.unwrap();
    let mut reply = Vec::new();
    socket.read_to_end(&mut reply).await.unwrap();
    String::from_utf8_lossy(&reply).to_string()
}

#[tokio::test]
async fn a_scripted_answer_is_sent_and_the_request_is_recorded_with_its_body() {
    let server = FakeHttp::start(|request| match request.path.as_str() {
        "/feed" => Answer::ok("text/calendar", "BEGIN").with_header("etag", "\"1\""),
        _ => Answer::status(404),
    })
    .await;
    let body = "{\"text\":\"hi\"}";
    let reply = send(
        &server,
        &format!(
            "POST /feed HTTP/1.1\r\nhost: x\r\ncontent-length: {}\r\n\r\n{body}",
            body.len()
        ),
    )
    .await;
    assert!(reply.starts_with("HTTP/1.1 200 "), "{reply}");
    assert!(reply.contains("content-type: text/calendar\r\n"), "{reply}");
    assert!(reply.contains("etag: \"1\"\r\n"), "{reply}");
    assert!(reply.ends_with("\r\n\r\nBEGIN"), "{reply}");
    assert!(
        send(&server, "GET /other HTTP/1.1\r\n\r\n")
            .await
            .starts_with("HTTP/1.1 404 ")
    );
    let requests = server.requests();
    assert_eq!(server.hits(), 2);
    assert_eq!(
        (requests[0].method.as_str(), requests[0].path.as_str()),
        ("POST", "/feed")
    );
    assert_eq!(requests[0].body, body);
    assert_eq!(requests[0].header("Content-Length"), Some("13"));
}

#[tokio::test]
async fn raw_bytes_go_out_as_they_are_and_a_delay_holds_the_answer_back() {
    let garbage = FakeHttp::always(Answer::Raw(b"not http\r\n\r\n".to_vec())).await;
    assert_eq!(
        send(&garbage, "GET / HTTP/1.1\r\n\r\n").await,
        "not http\r\n\r\n"
    );
    let slow = FakeHttp::always(Answer::status(204).after(Duration::from_millis(150))).await;
    let started = Instant::now();
    assert!(
        send(&slow, "GET / HTTP/1.1\r\n\r\n")
            .await
            .starts_with("HTTP/1.1 204 ")
    );
    assert!(started.elapsed() >= Duration::from_millis(150));
}

#[tokio::test]
async fn a_hanging_server_never_answers() {
    let server = FakeHttp::always(Answer::Hang).await;
    let waited = tokio::time::timeout(
        Duration::from_millis(200),
        send(&server, "GET / HTTP/1.1\r\n\r\n"),
    )
    .await;
    assert!(waited.is_err());
}

#[tokio::test]
async fn pages_answer_by_path_and_anything_else_is_not_found() {
    let server = FakeHttp::pages(std::collections::BTreeMap::from([(
        "/icon.png".to_string(),
        Answer::ok("image/png", b"PNG".to_vec()),
    )]))
    .await;
    assert!(
        send(&server, "GET /icon.png HTTP/1.1\r\n\r\n")
            .await
            .ends_with("PNG")
    );
    assert!(
        send(&server, "GET /other HTTP/1.1\r\n\r\n")
            .await
            .starts_with("HTTP/1.1 404 ")
    );
}
