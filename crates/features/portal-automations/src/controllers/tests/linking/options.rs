use axum::body::Body;
use axum::http::header::CONTENT_TYPE;
use axum::http::{Request, StatusCode};

use super::setup::{SAFE_SYNC, SYNC, call, finished, ready};
use crate::controllers::tests::automations::send;

#[tokio::test]
async fn an_option_smuggled_through_a_variable_is_refused() {
    let api = ready();
    let run_id = call(
        &api,
        SYNC,
        "application/json",
        r#"{"host":"-oProxyCommand=sh"}"#,
    )
    .await
    .unwrap();
    let body = finished(&api, run_id).await;
    assert_eq!(body["outcome"]["result"], "refused", "{body}");
    assert!(
        body["outcome"]["reason"]
            .as_str()
            .unwrap()
            .contains("put -- before it"),
        "{body}"
    );
}

#[tokio::test]
async fn a_separator_makes_the_same_value_safe() {
    let api = ready();
    let run_id = call(
        &api,
        SAFE_SYNC,
        "application/json",
        r#"{"host":"-oProxyCommand=sh"}"#,
    )
    .await
    .unwrap();
    let body = finished(&api, run_id).await;
    assert_eq!(body["outcome"]["result"], "succeeded", "{body}");
    assert_eq!(body["outcome"]["stdout"]["tail"], "-- -oProxyCommand=sh\n");
}

#[tokio::test]
async fn a_variable_holding_a_nul_is_refused_with_422() {
    let api = ready();
    let request = Request::post(format!("/webhook/{SYNC}"))
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"host":"nas\u0000"}"#))
        .unwrap();
    let (status, _, body) = send(&api, request).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["errors"][0]["field"], "host");
}
