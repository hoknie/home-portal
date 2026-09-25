use std::collections::BTreeMap;
use std::fs;
use std::sync::Arc;

use axum::body::Body;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE, ETAG, IF_NONE_MATCH};
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use portal_config::ConfigStore;
use portal_feature::Feature;
use serde_json::Value;
use time::macros::datetime;
use tower::ServiceExt;

use super::IconsFeature;
use crate::fakes::{Reply, Site};

const PNG: &[u8] = b"\x89PNG\r\n\x1a\n and some pixels";

#[tokio::test]
async fn the_portal_serves_the_icon_it_fetched_and_answers_304_to_the_same_tag() {
    let site = Site::start(BTreeMap::from([(
        "/icon.png".to_string(),
        Reply::image(PNG),
    )]))
    .await;
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    fs::write(
        &path,
        format!(
            "[[services]]\nid = \"nas\"\nname = \"NAS\"\nurl = \"{}\"\nicon = \"url:{}/icon.png\"\n",
            site.url(),
            site.url()
        ),
    )
    .unwrap();
    let store = Arc::new(ConfigStore::open(&path).unwrap());
    let feature = IconsFeature::new(store, "http://unused.invalid").unwrap();
    feature
        .icons()
        .refresh_all(datetime!(2026-09-22 10:00 UTC))
        .await;
    let router = feature.router();

    let response = router
        .clone()
        .oneshot(Request::get("/api/icons/nas").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let tag = response.headers()[ETAG].to_str().unwrap().to_string();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&bytes[..], PNG);

    let again = router
        .clone()
        .oneshot(
            Request::get("/api/icons/nas")
                .header(IF_NONE_MATCH, &tag)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(again.status(), StatusCode::NOT_MODIFIED);
    assert!(
        again
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .is_empty()
    );

    let missing = router
        .oneshot(Request::get("/api/icons/nope").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}

async fn preview(site: &Site, body: &str) -> (StatusCode, String, String, Vec<u8>, bool) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    fs::write(&path, "").unwrap();
    let store = Arc::new(ConfigStore::open(&path).unwrap());
    let feature = IconsFeature::new(store, &format!("{}/catalog", site.url())).unwrap();
    let response = feature
        .router()
        .oneshot(
            Request::post(IconsFeature::PREVIEW_PATH)
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body.replace("SITE", &site.url())))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let header = |name| {
        response
            .headers()
            .get(name)
            .map(|value: &axum::http::HeaderValue| value.to_str().unwrap().to_string())
            .unwrap_or_default()
    };
    let content_type = header(CONTENT_TYPE);
    let cache = header(CACHE_CONTROL);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let cached = fs::read_dir(directory.path().join("icons"))
        .map(|mut entries| entries.next().is_some())
        .unwrap_or(false);
    (status, content_type, cache, bytes.to_vec(), cached)
}

fn field_error(bytes: &[u8]) -> (String, String) {
    let body: Value = serde_json::from_slice(bytes).unwrap();
    (
        body["errors"][0]["field"].as_str().unwrap().to_string(),
        body["errors"][0]["message"].as_str().unwrap().to_string(),
    )
}

#[tokio::test]
async fn a_catalogue_icon_is_previewed_without_being_cached() {
    let site = Site::start(BTreeMap::from([(
        "/catalog/jellyfin.png".to_string(),
        Reply::image(PNG),
    )]))
    .await;
    let (status, content_type, cache, bytes, cached) =
        preview(&site, r#"{"icon":"catalog:jellyfin","url":null}"#).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(content_type, "image/png");
    assert_eq!(cache, "no-store");
    assert_eq!(bytes, PNG);
    assert!(!cached, "a preview writes nothing to the icons directory");
}

#[tokio::test]
async fn auto_finds_the_icon_at_the_address_it_is_given() {
    let site = Site::start(BTreeMap::from([
        (
            "/".to_string(),
            Reply::page("<html><head><link rel=\"icon\" href=\"/page.png\"></head></html>"),
        ),
        ("/page.png".to_string(), Reply::image(PNG)),
    ]))
    .await;
    let (status, _, _, bytes, cached) = preview(&site, r#"{"icon":"auto","url":"SITE/"}"#).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(bytes, PNG);
    assert!(!cached);
}

#[tokio::test]
async fn a_page_that_is_not_an_image_is_refused_on_the_icon_field() {
    let site = Site::start(BTreeMap::from([(
        "/".to_string(),
        Reply::page("<html></html>"),
    )]))
    .await;
    let (status, _, _, bytes, _) = preview(&site, r#"{"icon":"url:SITE/","url":null}"#).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let (field, message) = field_error(&bytes);
    assert_eq!(field, "icon");
    assert!(message.contains("not an image"), "{message}");
}

#[tokio::test]
async fn a_preview_that_cannot_be_fetched_is_refused_on_the_icon_field() {
    let site = Site::start(BTreeMap::new()).await;
    for body in [
        r#"{"icon":"auto","url":""}"#,
        r#"{"icon":"auto","url":"ftp://nas.local"}"#,
        r#"{"icon":"auto"}"#,
        r#"{"icon":"film","url":null}"#,
        r#"{"icon":"sprite:nas","url":null}"#,
        r#"{"icon":"url:SITE/missing.png","url":null}"#,
    ] {
        let (status, _, _, bytes, _) = preview(&site, body).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        assert_eq!(field_error(&bytes).0, "icon", "{body}");
    }
}
