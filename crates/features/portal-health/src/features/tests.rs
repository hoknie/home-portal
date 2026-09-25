use axum::body::Body;
use axum::http::header::CONTENT_TYPE;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use portal_feature::Feature;
use tower::ServiceExt;

use super::HealthFeature;

#[tokio::test]
async fn the_liveness_endpoint_answers_ok_in_plain_text() {
    let request = Request::get(HealthFeature::PATH)
        .body(Body::empty())
        .unwrap();
    let response = HealthFeature
        .public_router()
        .oneshot(request)
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let content_type = response.headers()[CONTENT_TYPE].to_str().unwrap();
    assert!(content_type.starts_with("text/plain"));
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&bytes[..], b"ok");
}

#[test]
fn the_feature_is_named_after_its_crate() {
    assert_eq!(
        HealthFeature.name(),
        env!("CARGO_PKG_NAME").trim_start_matches("portal-")
    );
}
