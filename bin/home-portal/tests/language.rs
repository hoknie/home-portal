mod support;

use std::sync::Arc;

use axum::body::Body;
use axum::http::header::{CONTENT_LANGUAGE, LOCATION, VARY};
use axum::http::{HeaderMap, Request, StatusCode};
use axum::response::Response;
use axum::routing::get;
use axum::{Extension, Router};
use home_portal::{Registry, assemble};
use http_body_util::BodyExt;
use portal_feature::{ApiError, Feature, Gate, Principal};
use portal_model::Language;
use tower::ServiceExt;

const LANGUAGE_PATH: &str = "/api/language";

struct LanguageFeature;

impl Feature for LanguageFeature {
    fn name(&self) -> &'static str {
        "language"
    }

    fn router(&self) -> Router {
        Router::new()
    }

    fn public_router(&self) -> Router {
        Router::new().route(
            LANGUAGE_PATH,
            get(|Extension(language): Extension<Language>| async move { language.code() }),
        )
    }
}

struct Nobody;

impl Gate for Nobody {
    fn admit(&self, _headers: &HeaderMap) -> Result<Principal, ApiError> {
        Err(ApiError::Unauthorized)
    }
}

fn portal(extra: &str) -> Router {
    let directory = Box::leak(Box::new(tempfile::tempdir().unwrap()));
    let path = support::with_extra(directory, "secret", extra);
    let configuration = support::wiring_for(&path).configuration;
    assemble(&Registry {
        features: vec![Arc::new(LanguageFeature)],
        gate: Arc::new(Nobody),
        configuration: configuration.clone(),
        widgets: Arc::new(portal_widget::WidgetRegistry::new(
            configuration,
            Vec::new(),
        )),
    })
}

async fn send(router: Router, path: &str, headers: &[(&str, &str)]) -> Response {
    let mut request = Request::get(path);
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    router
        .oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap()
}

async fn decided(extra: &str, headers: &[(&str, &str)]) -> String {
    let response = send(portal(extra), LANGUAGE_PATH, headers).await;
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).unwrap()
}

const SPANISH_BROWSER: (&str, &str) = ("accept-language", "es-ES,es;q=0.9,en;q=0.8");

#[tokio::test]
async fn the_browser_decides_when_there_is_no_cookie() {
    assert_eq!(decided("", &[SPANISH_BROWSER]).await, "es");
}

#[tokio::test]
async fn the_cookie_wins_over_the_browser() {
    assert_eq!(
        decided(
            "",
            &[
                SPANISH_BROWSER,
                ("cookie", "theme=dark; portal_language=ru")
            ]
        )
        .await,
        "ru"
    );
}

#[tokio::test]
async fn a_cookie_naming_nothing_supported_leaves_it_to_the_browser() {
    assert_eq!(
        decided(
            "",
            &[("accept-language", "ru"), ("cookie", "portal_language=de")]
        )
        .await,
        "ru"
    );
}

#[tokio::test]
async fn without_the_section_an_unmatched_browser_gets_english() {
    assert_eq!(decided("", &[("accept-language", "de")]).await, "en");
}

#[tokio::test]
async fn the_configured_default_takes_over_when_the_browser_matches_nothing() {
    let extra = "\n[interface]\ndefault_language = \"es\"\n";
    assert_eq!(decided(extra, &[("accept-language", "de")]).await, "es");
    assert_eq!(decided(extra, &[]).await, "es");
}

#[tokio::test]
async fn a_deep_link_keeps_its_address_and_carries_its_language_when_the_interface_is_built() {
    let response = send(
        portal(""),
        "/admin/services/edit/?id=nas",
        &[("accept-language", "en")],
    )
    .await;
    assert!(response.headers().get(LOCATION).is_none());
    if response.status() == StatusCode::OK {
        assert_eq!(response.headers()[CONTENT_LANGUAGE], "en");
        assert_eq!(response.headers()[VARY], "Cookie, Accept-Language");
    } else {
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}

#[tokio::test]
async fn api_answers_do_not_vary_by_language() {
    let response = send(portal(""), LANGUAGE_PATH, &[SPANISH_BROWSER]).await;
    assert!(response.headers().get(VARY).is_none());
    assert!(response.headers().get(CONTENT_LANGUAGE).is_none());
}

#[test]
fn the_portal_serves_exactly_the_languages_the_interface_has_dictionaries_for() {
    let directory =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../web/src/shared/i18n/messages");
    let mut dictionaries: Vec<String> = std::fs::read_dir(&directory)
        .unwrap()
        .filter_map(|entry| {
            let name = entry.unwrap().file_name().into_string().unwrap();
            name.strip_suffix(".json").map(str::to_string)
        })
        .collect();
    dictionaries.sort();
    let mut served: Vec<String> = Language::ALL
        .iter()
        .map(|language| language.code().to_string())
        .collect();
    served.sort();
    assert_eq!(dictionaries, served);
}
