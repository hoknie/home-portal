use std::fmt;

use axum::http::header::IF_MATCH;
use axum::http::{HeaderMap, HeaderValue};
use portal_feature::ApiError;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Revision(String);

impl Revision {
    pub fn of(bytes: &[u8]) -> Revision {
        let digest = Sha256::digest(bytes);
        Revision(digest.iter().map(|byte| format!("{byte:02x}")).collect())
    }

    pub fn from_headers(headers: &HeaderMap) -> Result<Revision, ApiError> {
        let value = headers
            .get(IF_MATCH)
            .and_then(|value| value.to_str().ok())
            .ok_or(ApiError::PreconditionRequired)?;
        let unquoted = value.trim().trim_start_matches("W/").trim_matches('"');
        Ok(Revision(unquoted.to_string()))
    }

    pub fn etag(&self) -> HeaderValue {
        HeaderValue::from_str(&format!("\"{}\"", self.0))
            .expect("a hex digest is a valid header value")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Revision {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
