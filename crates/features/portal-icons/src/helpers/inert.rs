use axum::http::header::{CONTENT_DISPOSITION, CONTENT_SECURITY_POLICY, X_CONTENT_TYPE_OPTIONS};
use axum::http::{HeaderMap, HeaderValue};

pub const ICON_POLICY: &str = "default-src 'none'; style-src 'unsafe-inline'; sandbox";

pub fn inert_headers(headers: &mut HeaderMap, extension: &str) {
    headers.insert(
        CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(ICON_POLICY),
    );
    headers.insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    if let Ok(disposition) =
        HeaderValue::from_str(&format!("inline; filename=\"icon.{extension}\""))
    {
        headers.insert(CONTENT_DISPOSITION, disposition);
    }
}
