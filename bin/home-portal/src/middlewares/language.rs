use axum::body::Body;
use axum::extract::State;
use axum::http::header::ACCEPT_LANGUAGE;
use axum::http::{HeaderMap, Request};
use axum::middleware::Next;
use axum::response::Response;
use portal_model::Language;
use portal_web::CurrentInterface;

use super::cookie_value::cookie_value;

pub const LANGUAGE_COOKIE: &str = "portal_language";

pub async fn decide_language(
    State(interface): State<CurrentInterface>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let fallback = || interface.run().unwrap_or_default();
    let language = decided(request.headers(), fallback);
    with_language(request, language, next).await
}

pub async fn decide_language_from(
    State(fallback): State<Language>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let language = decided(request.headers(), || fallback);
    with_language(request, language, next).await
}

fn decided(headers: &HeaderMap, fallback: impl FnOnce() -> Language) -> Language {
    let chosen = cookie_value(headers, LANGUAGE_COOKIE).and_then(Language::parse);
    chosen.unwrap_or_else(|| {
        let accepted = headers
            .get(ACCEPT_LANGUAGE)
            .and_then(|value| value.to_str().ok());
        Language::negotiate(accepted, fallback())
    })
}

async fn with_language(mut request: Request<Body>, language: Language, next: Next) -> Response {
    request.extensions_mut().insert(language);
    next.run(request).await
}
