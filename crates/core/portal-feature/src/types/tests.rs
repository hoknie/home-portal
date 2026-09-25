use axum::http::StatusCode;
use axum::http::header::CONTENT_TYPE;
use axum::response::IntoResponse;
use http_body_util::BodyExt;

use axum::http::header::RETRY_AFTER;

use time::OffsetDateTime;

use super::{ApiError, EventName, FieldError, PortalEvent, StatusChange, Visitor};

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

fn built_by_its_constructor(name: EventName) -> PortalEvent {
    let at = OffsetDateTime::UNIX_EPOCH;
    let change = StatusChange {
        service: "nas".into(),
        name: "NAS".into(),
        was: "up".into(),
        now: "down".into(),
        error: Some("refused".into()),
        diagnosis: Some("refused".into()),
        notify: true,
    };
    match name {
        EventName::PortalStarted | EventName::PortalStopping => {
            PortalEvent::portal(name, "127.0.0.1:8080", at)
        }
        EventName::ServiceStatusChanged => PortalEvent::status_changed(&change, at),
        EventName::UserSignedIn | EventName::UserSignedOut | EventName::UserSignInFailed => {
            PortalEvent::visited(name, &Visitor::default(), at)
        }
        other => PortalEvent::of(other, at, &[]),
    }
}

#[test]
fn every_event_carries_exactly_its_own_fields_after_the_event_fields() {
    for name in EventName::ALL {
        let event = built_by_its_constructor(name);
        let keys: Vec<&str> = event.fields.iter().map(|(key, _)| *key).collect();
        let mut expected = vec![PortalEvent::NAME_FIELD, PortalEvent::AT_FIELD];
        expected.extend_from_slice(name.fields());
        assert_eq!(keys, expected, "{}", name.name());
        assert_eq!(event.value(PortalEvent::NAME_FIELD), Some(name.name()));
    }
}

#[test]
fn a_value_is_cut_to_its_limit_on_a_character_boundary_without_nul() {
    let huge = format!("\0{}", "é".repeat(600_000));
    let event = PortalEvent::visited(
        EventName::UserSignInFailed,
        &Visitor {
            user: huge,
            reason: Some(PortalEvent::CREDENTIALS),
            ..Visitor::default()
        },
        OffsetDateTime::UNIX_EPOCH,
    );
    let user = event.value("user.name").unwrap();
    assert!(user.len() <= PortalEvent::VALUE_LIMIT);
    assert!(user.len() > PortalEvent::VALUE_LIMIT - 2);
    assert!(!user.contains('\0'));
    assert_eq!(event.value("sign_in.reason"), Some("credentials"));
}

#[test]
fn a_missing_value_is_the_empty_string_and_an_unknown_name_is_unknown() {
    let event = PortalEvent::of(EventName::ServiceUpdated, OffsetDateTime::UNIX_EPOCH, &[]);
    assert_eq!(event.value("service.previous_id"), Some(""));
    assert_eq!(event.value("event.at"), Some("1970-01-01T00:00:00Z"));
    assert_eq!(EventName::from("nope"), EventName::Unknown);
    assert_eq!(EventName::from("user.signed-in"), EventName::UserSignedIn);
}

#[test]
fn webhook_variables_are_named_under_webhook_and_cleaned() {
    let event = PortalEvent::of(EventName::WebhookReceived, OffsetDateTime::UNIX_EPOCH, &[])
        .with_variables(&[("branch".into(), "ma\0in".into())]);
    assert_eq!(
        event.variables,
        vec![("webhook.branch".to_string(), "main".to_string())]
    );
    assert_eq!(EventName::from("manual"), EventName::Manual);
    assert!(EventName::Manual.fields().is_empty());
}
