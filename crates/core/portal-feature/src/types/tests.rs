use axum::http::StatusCode;
use axum::http::header::CONTENT_TYPE;
use axum::response::IntoResponse;
use http_body_util::BodyExt;

use axum::http::header::RETRY_AFTER;

use super::{ApiError, FieldError};

async fn answer(error: ApiError) -> (StatusCode, String, String) {
    let response = error.into_response();
    let status = response.status();
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        content_type,
        String::from_utf8(bytes.to_vec()).unwrap(),
    )
}

#[tokio::test]
async fn every_kind_of_error_answers_with_its_own_status() {
    let cases = [
        (ApiError::BadRequest("bad".into()), StatusCode::BAD_REQUEST),
        (ApiError::Unauthorized, StatusCode::UNAUTHORIZED),
        (ApiError::NotFound("missing"), StatusCode::NOT_FOUND),
        (ApiError::Conflict("stale".into()), StatusCode::CONFLICT),
        (
            ApiError::UnsupportedMediaType,
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ),
        (
            ApiError::PreconditionRequired,
            StatusCode::PRECONDITION_REQUIRED,
        ),
        (
            ApiError::invalid("id", "bad"),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            ApiError::TooManyRequests {
                retry_after_seconds: 60,
            },
            StatusCode::TOO_MANY_REQUESTS,
        ),
        (
            ApiError::BadGateway("upstream".into()),
            StatusCode::BAD_GATEWAY,
        ),
        (
            ApiError::ServiceUnavailable("away".into()),
            StatusCode::SERVICE_UNAVAILABLE,
        ),
        (
            ApiError::Internal("boom".into()),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    ];
    for (error, expected) in cases {
        assert_eq!(answer(error).await.0, expected);
    }
}

#[tokio::test]
async fn an_error_body_is_plain_text_that_describes_it() {
    let (_, content_type, body) = answer(ApiError::BadGateway("upstream refused".into())).await;
    assert!(content_type.starts_with("text/plain"));
    assert_eq!(body, "upstream refused");
}

#[tokio::test]
async fn an_internal_error_never_shows_its_diagnostic_to_the_client() {
    let (status, _, body) = answer(ApiError::Internal("password=hunter2".into())).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body, ApiError::INTERNAL_BODY);
    assert!(!body.contains("hunter2"));
}

#[tokio::test]
async fn invalid_input_answers_json_with_one_entry_per_field() {
    let error = ApiError::Invalid(vec![
        FieldError::new("id", "must start with a lower-case letter"),
        FieldError::new("url", "must be http or https"),
    ]);
    let (status, content_type, body) = answer(error).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(content_type.starts_with("application/json"));
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let fields: Vec<&str> = json["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|error| error["field"].as_str().unwrap())
        .collect();
    assert_eq!(fields, vec!["id", "url"]);
}

#[tokio::test]
async fn too_many_requests_says_when_to_retry() {
    let response = ApiError::TooManyRequests {
        retry_after_seconds: 42,
    }
    .into_response();
    assert_eq!(response.headers()[RETRY_AFTER], "42");
}
