use axum::http::HeaderMap;

use crate::types::{ApiError, Principal};

pub trait Gate: Send + Sync {
    fn admit(&self, headers: &HeaderMap) -> Result<Principal, ApiError>;
}
