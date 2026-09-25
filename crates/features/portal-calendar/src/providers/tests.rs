use std::fs;
use std::sync::Arc;

use portal_config::ConfigStore;
use portal_feature::WidgetProvider;
use serde_json::json;
use tempfile::TempDir;
use time::macros::datetime;

use super::CalendarProvider;
use crate::fakes::FeedService;
use crate::types::CalendarSettings;

const FEED: &str = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nSUMMARY:Dentist\r\nDTSTART:20260923T090000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

fn store_with(text: &str) -> (TempDir, Arc<ConfigStore>) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    fs::write(&path, text).unwrap();
    #[cfg(unix)]
    if text.contains("[secrets]") {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    }
    let store = Arc::new(ConfigStore::open(&path).unwrap());
    (directory, store)
}

fn settings(url: &str) -> serde_json::Value {
    json!({ "url": url, "days": 7, "limit": 10 })
}

#[tokio::test]
async fn the_events_of_a_feed_become_the_widget_data() {
    let upstream = FeedService::start(FEED.to_string()).await;
    let (_directory, store) = store_with("");
    let provider = CalendarProvider::new(store).unwrap();
    let parsed: CalendarSettings = serde_json::from_value(settings(&upstream.url())).unwrap();
    let events = provider
        .events(&parsed, datetime!(2026-09-22 08:00 UTC))
        .await
        .unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].summary, "Dentist");
}

#[tokio::test]
async fn a_named_secret_is_sent_as_a_bearer_token() {
    let upstream = FeedService::start(FEED.to_string()).await;
    let (_directory, store) = store_with("[secrets]\ncalendar_password = \"open-sesame\"\n");
    let provider = CalendarProvider::new(store).unwrap();
    let mut value = settings(&upstream.url());
    value["secret"] = json!("calendar_password");
    let parsed: CalendarSettings = serde_json::from_value(value).unwrap();
    provider
        .events(&parsed, datetime!(2026-09-22 08:00 UTC))
        .await
        .unwrap();
    let request = upstream.requests().first().cloned().unwrap_or_default();
    assert!(
        request
            .to_lowercase()
            .contains("authorization: bearer open-sesame"),
        "{request}"
    );
}

#[tokio::test]
async fn a_feed_larger_than_the_ceiling_is_refused_by_size() {
    let big = format!(
        "BEGIN:VCALENDAR\r\n{}\r\nEND:VCALENDAR\r\n",
        "X-PADDING:".to_string() + &"x".repeat(6 * 1024 * 1024)
    );
    let upstream = FeedService::start(big).await;
    let (_directory, store) = store_with("");
    let provider = CalendarProvider::new(store).unwrap();
    let parsed: CalendarSettings = serde_json::from_value(settings(&upstream.url())).unwrap();
    let problem = provider
        .events(&parsed, datetime!(2026-09-22 08:00 UTC))
        .await
        .unwrap_err();
    assert!(problem.to_string().contains("larger than"), "{problem}");
}

#[test]
fn the_settings_are_checked_field_by_field() {
    let (_directory, store) = store_with("");
    let provider = CalendarProvider::new(store).unwrap();
    let fields: Vec<String> = provider
        .check(&json!({ "url": "ftp://x", "days": 40, "limit": 0, "timezone": "Europe/Riga" }))
        .into_iter()
        .map(|error| error.field)
        .collect();
    assert_eq!(fields, vec!["url", "days", "limit", "timezone"]);
    assert!(
        provider
            .check(&settings("https://example.com/c.ics"))
            .is_empty()
    );
}

#[test]
fn a_secret_the_configuration_does_not_hold_is_named() {
    let (_directory, store) = store_with("");
    let provider = CalendarProvider::new(store).unwrap();
    let mut value = settings("https://example.com/c.ics");
    value["secret"] = json!("calendar_password");
    let errors = provider.check(&value);
    assert_eq!(errors[0].field, "secret");
    assert!(
        errors[0].message.contains("calendar_password"),
        "{}",
        errors[0].message
    );
}
