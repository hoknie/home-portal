use std::borrow::Cow;
use std::collections::HashMap;

use axum::http::StatusCode;
use axum::http::header::{CACHE_CONTROL, CONTENT_LANGUAGE, CONTENT_TYPE, VARY};
use axum::response::Response;
use http_body_util::BodyExt;

use portal_model::Language;

use super::answer;
use crate::ports::AssetSource;
use crate::types::Asset;

struct Files(HashMap<&'static str, (&'static str, &'static str)>);

impl AssetSource for Files {
    fn get(&self, path: &str) -> Option<Asset> {
        self.0.get(path).map(|(content_type, body)| Asset {
            bytes: Cow::Borrowed(body.as_bytes()),
            content_type: content_type.to_string(),
        })
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

fn built() -> Files {
    Files(HashMap::from([
        ("en/index.html", ("text/html", "home")),
        ("en/services/index.html", ("text/html", "services")),
        ("en/favicon.ico", ("image/x-icon", "icon")),
        ("es/index.html", ("text/html", "inicio")),
        ("es/services/index.html", ("text/html", "servicios")),
        (
            "es/services/__next.services.__PAGE__.txt",
            ("text/plain", "payload"),
        ),
        ("ru/index.html", ("text/html", "главная")),
        ("_next/static/chunk.js", ("text/javascript", "code")),
    ]))
}

async fn body(response: Response) -> String {
    String::from_utf8(
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap()
}

#[tokio::test]
async fn the_root_is_the_entry_page_without_caching() {
    let response = answer(&built(), Language::En, "/");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[CACHE_CONTROL], "no-cache");
    assert_eq!(body(response).await, "home");
}

#[tokio::test]
async fn a_route_with_its_own_page_is_served_that_page() {
    assert_eq!(
        body(answer(&built(), Language::En, "/services/")).await,
        "services"
    );
    assert_eq!(
        body(answer(&built(), Language::En, "/services")).await,
        "services"
    );
}

#[tokio::test]
async fn an_unknown_route_even_with_dots_is_the_entry_page() {
    let response = answer(&built(), Language::En, "/services/edit/nas.local");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body(response).await, "home");
}

#[tokio::test]
async fn build_assets_are_cached_forever_with_their_type() {
    let response = answer(&built(), Language::En, "/_next/static/chunk.js");
    assert_eq!(
        response.headers()[CACHE_CONTROL],
        "public, max-age=31536000, immutable"
    );
    assert_eq!(response.headers()[CONTENT_TYPE], "text/javascript");
}

#[tokio::test]
async fn a_missing_asset_is_not_found() {
    assert_eq!(
        answer(&built(), Language::En, "/_next/static/missing.js").status(),
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn a_binary_built_without_the_interface_says_so() {
    let response = answer(&Files(HashMap::new()), Language::En, "/");
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert!(body(response).await.contains("just web"));
}

#[tokio::test]
async fn every_page_comes_from_the_tree_of_its_language() {
    assert_eq!(
        body(answer(&built(), Language::Es, "/services/")).await,
        "servicios"
    );
    assert_eq!(body(answer(&built(), Language::Ru, "/")).await, "главная");
    let deep = answer(&built(), Language::Es, "/services/edit/nas.local");
    assert_eq!(deep.status(), StatusCode::OK);
    assert_eq!(deep.headers()[CONTENT_LANGUAGE], "es");
    assert_eq!(deep.headers()[VARY], "Cookie, Accept-Language");
    assert_eq!(body(deep).await, "inicio");
}

#[tokio::test]
async fn a_route_payload_is_served_from_the_same_tree() {
    let response = answer(
        &built(),
        Language::Es,
        "/services/__next.services.__PAGE__.txt",
    );
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body(response).await, "payload");
}

#[tokio::test]
async fn a_shared_asset_is_found_whatever_the_language_and_does_not_vary() {
    let response = answer(&built(), Language::Ru, "/_next/static/chunk.js");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[CACHE_CONTROL],
        "public, max-age=31536000, immutable"
    );
    assert!(response.headers().get(VARY).is_none());
}

#[tokio::test]
async fn a_route_a_language_lacks_falls_back_to_that_languages_entry_page() {
    assert_eq!(
        body(answer(&built(), Language::Ru, "/services/")).await,
        "главная"
    );
}
