use std::sync::Arc;

use axum::body::Body;
use axum::http::header::{CONTENT_TYPE, ETAG, IF_MATCH};
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use portal_feature::{Channel, Feature};
use serde_json::{Value, json};
use tower::ServiceExt;

use crate::fakes::FakeChannel;
use crate::features::NotificationFeature;
use crate::services::tests::store;

struct Api {
    _directory: tempfile::TempDir,
    feature: NotificationFeature,
    path: std::path::PathBuf,
}

fn api(text: &str, channel: FakeChannel) -> Api {
    let (directory, configuration) = store(text);
    let path = directory.path().join("home-portal.toml");
    let feature =
        NotificationFeature::new(configuration, vec![Arc::new(channel) as Arc<dyn Channel>])
            .unwrap();
    Api {
        _directory: directory,
        feature,
        path,
    }
}

async fn send(api: &Api, request: Request<Body>) -> (StatusCode, Option<String>, Value) {
    let response = api.feature.router().oneshot(request).await.unwrap();
    let status = response.status();
    let etag = response
        .headers()
        .get(ETAG)
        .map(|value| value.to_str().unwrap().to_string());
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = serde_json::from_slice(&bytes)
        .unwrap_or(Value::String(String::from_utf8_lossy(&bytes).to_string()));
    (status, etag, body)
}

fn write(method: &str, uri: &str, revision: Option<&str>, body: &str) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(CONTENT_TYPE, "application/json");
    if let Some(revision) = revision {
        builder = builder.header(IF_MATCH, revision);
    }
    builder.body(Body::from(body.to_string())).unwrap()
}

fn get(uri: &str) -> Request<Body> {
    Request::get(uri).body(Body::empty()).unwrap()
}

#[tokio::test]
async fn the_list_carries_the_rules_and_each_channel_with_the_revision() {
    let api = api(
        "[notifications]\nstates = [\"down\"]\n",
        FakeChannel::ready("telegram"),
    );
    let (status, etag, body) = send(&api, get(NotificationFeature::COLLECTION)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(etag.is_some());
    assert_eq!(body["enabled"], true);
    assert_eq!(
        body["rules"],
        json!({"states": ["down"], "recovered": true})
    );
    assert_eq!(body["channels"][0]["name"], "telegram");
    assert_eq!(body["channels"][0]["readiness"], "ready");
    assert_eq!(body["channels"][0]["queued"], 0);
}

#[tokio::test]
async fn changing_the_rules_needs_the_revision_and_names_a_bad_state() {
    let api = api("# keep me\n", FakeChannel::ready("telegram"));
    let (status, _, _) = send(
        &api,
        write(
            "PUT",
            NotificationFeature::COLLECTION,
            None,
            r#"{"states":["down"],"recovered":false}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::PRECONDITION_REQUIRED);
    let (_, etag, _) = send(&api, get(NotificationFeature::COLLECTION)).await;
    let (status, _, body) = send(
        &api,
        write(
            "PUT",
            NotificationFeature::COLLECTION,
            etag.as_deref(),
            r#"{"states":["down","broken"],"recovered":false}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["errors"][0]["field"], "notifications.states");
    let (status, _, body) = send(
        &api,
        write(
            "PUT",
            NotificationFeature::COLLECTION,
            etag.as_deref(),
            r#"{"states":["down"],"recovered":false}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["rules"]["recovered"], false);
    let text = std::fs::read_to_string(&api.path).unwrap();
    assert!(
        text.contains("# keep me") && text.contains("recovered = false"),
        "{text}"
    );
}

#[tokio::test]
async fn a_channel_is_changed_by_name_and_an_unknown_one_is_not_found() {
    let api = api("", FakeChannel::ready("telegram"));
    let (_, etag, _) = send(&api, get(NotificationFeature::COLLECTION)).await;
    let (status, _, _) = send(
        &api,
        write(
            "PUT",
            "/api/notifications/channels/telegram",
            etag.as_deref(),
            r#"{"enabled":true}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _, _) = send(
        &api,
        write(
            "PUT",
            "/api/notifications/channels/sms",
            etag.as_deref(),
            "{}",
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_test_message_answers_its_delivery_and_is_refused_while_off_or_unconfigured() {
    let api = api("", FakeChannel::ready("telegram"));
    let (status, _, body) = send(
        &api,
        write(
            "POST",
            NotificationFeature::TEST,
            None,
            r#"{"channel":"telegram"}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["delivered"], true);
    let (_, _, listed) = send(&api, get(NotificationFeature::COLLECTION)).await;
    assert_eq!(listed["channels"][0]["last_delivery"]["delivered"], true);
    let off = self::api(
        "[modules]\nnotifications = false\n",
        FakeChannel::ready("telegram"),
    );
    let (status, _, _) = send(
        &off,
        write(
            "POST",
            NotificationFeature::TEST,
            None,
            r#"{"channel":"telegram"}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    let unconfigured = self::api(
        "",
        FakeChannel {
            ready: false,
            ..FakeChannel::ready("telegram")
        },
    );
    let (status, _, _) = send(
        &unconfigured,
        write(
            "POST",
            NotificationFeature::TEST,
            None,
            r#"{"channel":"telegram"}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}
