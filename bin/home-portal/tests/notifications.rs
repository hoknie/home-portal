use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::header::CONTENT_TYPE;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use portal_config::ConfigStore;
use portal_feature::{Channel, Feature};
use portal_notification::NotificationFeature;
use portal_telegram::TelegramChannel;
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tower::ServiceExt;

async fn bot_api() -> (String, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let requests = seen.clone();
    tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            let mut buffer = [0u8; 4096];
            let read = socket.read(&mut buffer).await.unwrap_or_default();
            requests
                .lock()
                .unwrap()
                .push(String::from_utf8_lossy(&buffer[..read]).to_string());
            let body = "{\"ok\":true}";
            let reply = format!(
                "HTTP/1.1 200 OK\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = socket.write_all(reply.as_bytes()).await;
        }
    });
    (format!("http://{address}"), seen)
}

async fn body_of(response: axum::response::Response) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn a_test_message_reaches_the_chat_and_no_answer_carries_the_token() {
    let (endpoint, requests) = bot_api().await;
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    fs::write(
        &path,
        "[secrets]\ntelegram_token = \"123456789:SECRET\"\n\n[notifications.telegram]\nenabled = true\nsecret = \"telegram_token\"\nchat_id = \"42\"\n",
    )
    .unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    let configuration = Arc::new(ConfigStore::open(&path).unwrap());
    let channel: Arc<dyn Channel> = Arc::new(TelegramChannel::new(&endpoint).unwrap());
    let feature = NotificationFeature::new(configuration, vec![channel]).unwrap();
    let response = feature
        .router()
        .oneshot(
            Request::post(NotificationFeature::TEST)
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"channel":"telegram"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let answer = body_of(response).await;
    assert_eq!(answer["delivered"], true, "{answer}");
    let sent = requests.lock().unwrap().join("\n");
    assert!(sent.contains("/bot123456789:SECRET/sendMessage"), "{sent}");
    assert!(sent.contains("\"chat_id\":\"42\""), "{sent}");
    let listed = body_of(
        feature
            .router()
            .oneshot(
                Request::get(NotificationFeature::COLLECTION)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(listed["channels"][0]["last_delivery"]["delivered"], true);
    assert_eq!(
        listed["channels"][0]["settings"]["secret"],
        "telegram_token"
    );
    assert!(!listed.to_string().contains("SECRET"), "{listed}");
    assert!(!answer.to_string().contains("SECRET"), "{answer}");
}
