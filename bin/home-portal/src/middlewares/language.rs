use std::sync::Arc;

use axum::body::Body;
use axum::extract::State;
use axum::http::Request;
use axum::http::header::ACCEPT_LANGUAGE;
use axum::middleware::Next;
use axum::response::Response;
use portal_config::ConfigStore;
use portal_model::Language;

use super::cookie_value::cookie_value;

pub const LANGUAGE_COOKIE: &str = "portal_language";

pub async fn decide_language(
    State(configuration): State<Arc<ConfigStore>>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let headers = request.headers();
    let chosen = cookie_value(headers, LANGUAGE_COOKIE).and_then(Language::parse);
    let language = chosen.unwrap_or_else(|| {
        let fallback =
            portal_web::read_interface(&configuration.read().document).unwrap_or_default();
        let accepted = headers
            .get(ACCEPT_LANGUAGE)
            .and_then(|value| value.to_str().ok());
        Language::negotiate(accepted, fallback)
    });
    request.extensions_mut().insert(language);
    next.run(request).await
}
