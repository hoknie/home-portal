use std::collections::BTreeMap;
use std::fs;
use std::sync::Arc;

use portal_config::ConfigStore;
use tempfile::TempDir;
use time::Duration;
use time::macros::datetime;
use toml_edit::DocumentMut;
use url::Url;

use super::{Icons, validate_icons};
use crate::clients::{Fetcher, discover_icon};
use crate::fakes::{Reply, Site};
use crate::types::IconSource;

const PNG: &[u8] = b"\x89PNG\r\n\x1a\n and some pixels";

fn portal(text: &str) -> (TempDir, Arc<ConfigStore>) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    fs::write(&path, text).unwrap();
    (directory, Arc::new(ConfigStore::open(&path).unwrap()))
}

fn service_with(icon: &str, url: &str) -> String {
    format!("[[services]]\nid = \"nas\"\nname = \"NAS\"\nurl = \"{url}\"\nicon = \"{icon}\"\n")
}

#[test]
fn an_icon_source_is_read_by_its_prefix() {
    assert_eq!(
        IconSource::parse("film").unwrap(),
        IconSource::Lucide("film".into())
    );
    assert_eq!(
        IconSource::parse("lucide:server").unwrap(),
        IconSource::Lucide("server".into())
    );
    assert_eq!(
        IconSource::parse("catalog:jellyfin").unwrap(),
        IconSource::Catalog("jellyfin".into())
    );
    assert_eq!(IconSource::parse("auto").unwrap(), IconSource::Discovered);
    assert!(matches!(
        IconSource::parse("file:/srv/icon.png").unwrap(),
        IconSource::File(_)
    ));
    assert!(matches!(
        IconSource::parse("url:https://x/icon.png").unwrap(),
        IconSource::Address(_)
    ));
}

#[test]
fn an_unknown_prefix_names_the_service_it_came_from() {
    let document: DocumentMut = service_with("sprite:nas", "http://nas.local")
        .parse()
        .unwrap();
    let errors = validate_icons(&document);
    assert_eq!(errors[0].field, "services[0].icon");
    assert!(errors[0].message.contains("nas"), "{}", errors[0].message);
    assert!(
        errors[0].message.contains("lucide:"),
        "{}",
        errors[0].message
    );
    let good: DocumentMut = service_with("auto", "http://nas.local").parse().unwrap();
    assert!(validate_icons(&good).is_empty());
}

#[tokio::test]
async fn an_icon_is_fetched_cached_and_reused_after_a_restart() {
    let site = Site::start(BTreeMap::from([(
        "/icon.png".to_string(),
        Reply::image(PNG),
    )]))
    .await;
    let (directory, store) = portal(&service_with(
        &format!("url:{}/icon.png", site.url()),
        &site.url(),
    ));
    let icons = Icons::new(store.clone(), "http://unused.invalid").unwrap();
    let now = datetime!(2026-09-22 10:00 UTC);
    icons.refresh_all(now).await;
    let stored = icons.icon_of("nas").unwrap();
    assert_eq!(stored.bytes, PNG);
    assert_eq!(stored.content_type, "image/png");
    assert!(directory.path().join("icons").join("index.json").is_file());

    let again = Icons::new(store, "http://unused.invalid").unwrap();
    assert_eq!(again.icon_of("nas").unwrap().bytes, PNG);
    let before = site.hits();
    again.refresh_all(now + Duration::days(1)).await;
    assert_eq!(site.hits(), before, "a fresh icon is not fetched again");
    again.refresh_all(now + Duration::days(8)).await;
    assert!(site.hits() > before, "after seven days it is fetched again");
}

#[tokio::test]
async fn something_that_is_not_an_image_leaves_the_previous_icon_in_place() {
    let good = Site::start(BTreeMap::from([(
        "/icon.png".to_string(),
        Reply::image(PNG),
    )]))
    .await;
    let (_directory, store) = portal(&service_with(
        &format!("url:{}/icon.png", good.url()),
        &good.url(),
    ));
    let icons = Icons::new(store, "http://unused.invalid").unwrap();
    let now = datetime!(2026-09-22 10:00 UTC);
    icons.refresh_all(now).await;
    let html = Site::start(BTreeMap::from([(
        "/icon.png".to_string(),
        Reply::page("<html></html>"),
    )]))
    .await;
    let problem = icons
        .refresh("nas", &format!("url:{}/icon.png", html.url()), now)
        .await
        .unwrap_err();
    assert!(problem.contains("not an image"), "{problem}");
    assert_eq!(icons.icon_of("nas").unwrap().bytes, PNG);
}

#[tokio::test]
async fn an_icon_larger_than_the_ceiling_is_refused_by_size() {
    let big = vec![b'x'; 600 * 1024];
    let mut body = b"\x89PNG\r\n\x1a\n".to_vec();
    body.extend_from_slice(&big);
    let site = Site::start(BTreeMap::from([(
        "/icon.png".to_string(),
        Reply {
            status: 200,
            content_type: "image/png",
            body,
        },
    )]))
    .await;
    let (_directory, store) = portal(&service_with(
        &format!("url:{}/icon.png", site.url()),
        &site.url(),
    ));
    let icons = Icons::new(store, "http://unused.invalid").unwrap();
    let problem = icons
        .refresh(
            "nas",
            &format!("url:{}/icon.png", site.url()),
            datetime!(2026-09-22 10:00 UTC),
        )
        .await
        .unwrap_err();
    assert!(problem.contains("larger than"), "{problem}");
    assert!(icons.icon_of("nas").is_none());
}

#[tokio::test]
async fn a_catalogue_icon_is_fetched_once_and_then_reused() {
    let catalog = Site::start(BTreeMap::from([(
        "/jellyfin.png".to_string(),
        Reply::image(PNG),
    )]))
    .await;
    let (_directory, store) = portal(&service_with("catalog:jellyfin", "http://nas.local"));
    let icons = Icons::new(store, &catalog.url()).unwrap();
    let now = datetime!(2026-09-22 10:00 UTC);
    icons.refresh_all(now).await;
    icons.refresh_all(now).await;
    assert_eq!(icons.icon_of("nas").unwrap().bytes, PNG);
    assert_eq!(catalog.hits(), 1);
}

#[tokio::test]
async fn a_service_icon_is_found_in_the_manifest_then_the_page_then_the_favicon() {
    let manifest = r#"{"icons":[{"src":"/small.png","sizes":"48x48"},{"src":"/large.png","sizes":"512x512"}]}"#;
    let site = Site::start(BTreeMap::from([
        (
            "/".to_string(),
            Reply::page("<html><head><link rel=\"manifest\" href=\"/app.webmanifest\"><link rel=\"icon\" href=\"/page.png\"></head></html>"),
        ),
        (
            "/app.webmanifest".to_string(),
            Reply {
                status: 200,
                content_type: "application/manifest+json",
                body: manifest.as_bytes().to_vec(),
            },
        ),
    ]))
    .await;
    let fetcher = Fetcher::new().unwrap();
    let found = discover_icon(&fetcher, &Url::parse(&site.url()).unwrap())
        .await
        .unwrap();
    assert!(found.path().ends_with("/large.png"), "{found}");

    let page_only = Site::start(BTreeMap::from([(
        "/".to_string(),
        Reply::page("<html><head><link rel='apple-touch-icon' href='/touch.png'></head></html>"),
    )]))
    .await;
    let found = discover_icon(&fetcher, &Url::parse(&page_only.url()).unwrap())
        .await
        .unwrap();
    assert!(found.path().ends_with("/touch.png"), "{found}");

    let bare = Site::start(BTreeMap::new()).await;
    let found = discover_icon(&fetcher, &Url::parse(&bare.url()).unwrap()).await;
    assert!(found.is_err() || found.unwrap().path() == "/favicon.ico");
}

#[tokio::test]
async fn a_service_whose_icon_is_nowhere_keeps_no_icon_and_says_why() {
    let site = Site::start(BTreeMap::from([(
        "/".to_string(),
        Reply::page("<html></html>"),
    )]))
    .await;
    let (_directory, store) = portal(&service_with("auto", &site.url()));
    let icons = Icons::new(store, "http://unused.invalid").unwrap();
    let problem = icons
        .refresh("nas", "auto", datetime!(2026-09-22 10:00 UTC))
        .await
        .unwrap_err();
    assert!(!problem.is_empty());
    assert!(icons.icon_of("nas").is_none());
    let described = icons.described();
    assert_eq!(described[0].service, "nas");
    assert!(!described[0].available);
}
