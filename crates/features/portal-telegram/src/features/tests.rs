use std::fs;
use std::sync::Arc;
use std::time::{Duration, Instant};

use portal_config::ConfigStore;
use portal_feature::{Feature, StatusChange};
use tempfile::TempDir;

use super::TelegramFeature;
use crate::fakes::BotService;

const ENABLED: &str = "[secrets]\ntelegram_token = \"abc\"\n\n[notifications.telegram]\nenabled = true\nsecret = \"telegram_token\"\nchat_id = \"42\"\n";

fn store(text: &str) -> (TempDir, Arc<ConfigStore>) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("home-portal.toml");
    fs::write(&path, text).unwrap();
    #[cfg(unix)]
    if text.contains("[secrets]") {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    }
    (directory, Arc::new(ConfigStore::open(&path).unwrap()))
}

fn change() -> StatusChange {
    StatusChange {
        service: "nas".into(),
        name: "NAS".into(),
        was: "up".into(),
        now: "down".into(),
        error: Some("connection refused".into()),
        diagnosis: None,
        notify: true,
    }
}

#[tokio::test]
async fn a_message_reaches_telegram_with_the_token_and_the_chat() {
    let telegram = BotService::start(200, Duration::ZERO).await;
    let (_directory, configuration) = store(ENABLED);
    let feature = TelegramFeature::new(configuration, &telegram.endpoint()).unwrap();
    for task in feature.loops() {
        tokio::spawn(task);
    }
    portal_feature::StatusObserver::changed(feature.observer().as_ref(), &change());
    for _ in 0..50 {
        if !telegram.requests().is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let request = telegram.requests().first().cloned().unwrap_or_default();
    assert!(request.contains("/botabc/sendMessage"), "{request}");
    assert!(request.contains("\"chat_id\":\"42\""), "{request}");
    assert!(request.contains("NAS"), "{request}");
}

#[tokio::test]
async fn a_telegram_that_never_answers_does_not_hold_up_the_portal() {
    let telegram = BotService::start(200, Duration::from_secs(30)).await;
    let (_directory, configuration) = store(ENABLED);
    let feature = TelegramFeature::new(configuration, &telegram.endpoint()).unwrap();
    for task in feature.loops() {
        tokio::spawn(task);
    }
    let observer = feature.observer();
    let started = Instant::now();
    for _ in 0..20 {
        portal_feature::StatusObserver::changed(observer.as_ref(), &change());
    }
    let taken = started.elapsed();
    assert!(
        taken < Duration::from_millis(100),
        "reporting changes took {taken:?}"
    );
    assert!(feature.outbox().waiting() > 0);
}

#[test]
fn a_portal_with_telegram_enabled_and_nothing_to_send_with_refuses_to_start() {
    let (_directory, configuration) = store("[notifications.telegram]\nenabled = true\n");
    let outcome = TelegramFeature::new(configuration, "http://telegram.invalid");
    let problem = match outcome {
        Err(problem) => problem,
        Ok(_) => panic!("a portal with telegram enabled and no token must refuse to start"),
    };
    assert!(problem.contains("secret"), "{problem}");
}
