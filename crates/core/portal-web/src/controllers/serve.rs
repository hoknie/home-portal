use axum::Extension;
use axum::http::header::{CACHE_CONTROL, CONTENT_LANGUAGE, CONTENT_TYPE, VARY};
use axum::http::{HeaderValue, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use portal_model::Language;

use crate::assets::Embedded;
use crate::helpers::looks_like_asset;
use crate::ports::AssetSource;
use crate::types::Asset;

pub const ENTRY_PAGE: &str = "index.html";
pub const IMMUTABLE_PREFIX: &str = "_next/static/";
pub const IMMUTABLE: &str = "public, max-age=31536000, immutable";
pub const NO_CACHE: &str = "no-cache";
pub const NOT_BUILT: &str =
    "the interface is not built into this binary; run `just web` and rebuild";
pub const NOT_FOUND: &str = "not found";
pub const SHARED_PREFIX: &str = "_next/";
pub const VARIES_BY: &str = "Cookie, Accept-Language";

pub async fn serve(language: Option<Extension<Language>>, uri: Uri) -> Response {
    let language = language
        .map(|Extension(language)| language)
        .unwrap_or_default();
    answer(&Embedded, language, uri.path())
}

pub fn answer(source: &dyn AssetSource, language: Language, path: &str) -> Response {
    if source.is_empty() {
        return (StatusCode::SERVICE_UNAVAILABLE, NOT_BUILT).into_response();
    }
    let path = path.trim_start_matches('/');
    if path.starts_with(SHARED_PREFIX) {
        return match source.get(path) {
            Some(asset) => file(path, asset),
            None => (StatusCode::NOT_FOUND, NOT_FOUND).into_response(),
        };
    }
    let tree = language.code();
    let exact = format!("{tree}/{path}");
    let nested = format!("{tree}/{}/{ENTRY_PAGE}", path.trim_end_matches('/')).replace("//", "/");
    let found = (!path.is_empty())
        .then(|| source.get(&exact).map(|asset| (exact.clone(), asset)))
        .flatten()
        .or_else(|| source.get(&nested).map(|asset| (nested.clone(), asset)));
    if let Some((served, asset)) = found {
        return in_language(file(&served, asset), language);
    }
    if looks_like_asset(path) {
        return (StatusCode::NOT_FOUND, NOT_FOUND).into_response();
    }
    match source.get(&format!("{tree}/{ENTRY_PAGE}")) {
        Some(asset) => in_language(file(ENTRY_PAGE, asset), language),
        None => (StatusCode::SERVICE_UNAVAILABLE, NOT_BUILT).into_response(),
    }
}

fn in_language(mut response: Response, language: Language) -> Response {
    let headers = response.headers_mut();
    headers.insert(CONTENT_LANGUAGE, HeaderValue::from_static(language.code()));
    headers.insert(VARY, HeaderValue::from_static(VARIES_BY));
    response
}

fn file(path: &str, asset: Asset) -> Response {
    let cache = if path.starts_with(IMMUTABLE_PREFIX) {
        IMMUTABLE
    } else {
        NO_CACHE
    };
    let mut response = (StatusCode::OK, asset.bytes.into_owned()).into_response();
    let headers = response.headers_mut();
    if let Ok(content_type) = HeaderValue::from_str(&asset.content_type) {
        headers.insert(CONTENT_TYPE, content_type);
    }
    headers.insert(CACHE_CONTROL, HeaderValue::from_static(cache));
    response
}
