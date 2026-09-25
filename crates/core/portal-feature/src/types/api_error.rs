use axum::Json;
use axum::http::header::RETRY_AFTER;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use serde_json::json;

use super::FieldError;

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    Unauthorized,
    NotFound(&'static str),
    Conflict(String),
    UnsupportedMediaType,
    PreconditionRequired,
    Invalid(Vec<FieldError>),
    TooManyRequests { retry_after_seconds: u64 },
    BadGateway(String),
    ServiceUnavailable(String),
    Internal(String),
}

impl ApiError {
    pub const INTERNAL_BODY: &'static str = "internal error";
    pub const UNAUTHORIZED_BODY: &'static str = "sign in required";
    pub const UNSUPPORTED_MEDIA_TYPE_BODY: &'static str = "request body must be application/json";
    pub const PRECONDITION_REQUIRED_BODY: &'static str =
        "If-Match with the configuration revision is required";
    pub const TOO_MANY_REQUESTS_BODY: &'static str = "too many attempts, try again later";

    pub fn status(&self) -> StatusCode {
        match self {
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::Conflict(_) => StatusCode::CONFLICT,
            ApiError::UnsupportedMediaType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ApiError::PreconditionRequired => StatusCode::PRECONDITION_REQUIRED,
            ApiError::Invalid(_) => StatusCode::UNPROCESSABLE_ENTITY,
            ApiError::TooManyRequests { .. } => StatusCode::TOO_MANY_REQUESTS,
            ApiError::BadGateway(_) => StatusCode::BAD_GATEWAY,
            ApiError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn invalid(field: &str, message: &str) -> ApiError {
        ApiError::Invalid(vec![FieldError::new(field, message)])
    }

    fn text(self) -> String {
        match self {
            ApiError::BadRequest(message)
            | ApiError::Conflict(message)
            | ApiError::BadGateway(message)
            | ApiError::ServiceUnavailable(message) => message,
            ApiError::NotFound(what) => what.to_string(),
            ApiError::Unauthorized => Self::UNAUTHORIZED_BODY.to_string(),
            ApiError::UnsupportedMediaType => Self::UNSUPPORTED_MEDIA_TYPE_BODY.to_string(),
            ApiError::PreconditionRequired => Self::PRECONDITION_REQUIRED_BODY.to_string(),
            ApiError::TooManyRequests { .. } => Self::TOO_MANY_REQUESTS_BODY.to_string(),
            ApiError::Invalid(_) => String::new(),
            ApiError::Internal(diagnostic) => {
                tracing::error!(%diagnostic, "internal error");
                Self::INTERNAL_BODY.to_string()
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status();
        match self {
            ApiError::Invalid(errors) => {
                (status, Json(json!({ "errors": errors }))).into_response()
            }
            ApiError::TooManyRequests {
                retry_after_seconds,
            } => {
                let mut response = (status, Self::TOO_MANY_REQUESTS_BODY).into_response();
                response
                    .headers_mut()
                    .insert(RETRY_AFTER, HeaderValue::from(retry_after_seconds));
                response
            }
            other => (status, other.text()).into_response(),
        }
    }
}
