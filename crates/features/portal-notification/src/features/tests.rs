use std::sync::Arc;
use std::time::{Duration, Instant};

use portal_feature::{Channel, FieldError, Notification, StatusChange};

use super::NotificationFeature;
use crate::fakes::FakeChannel;
use crate::services::tests::{change, store};

fn feature(channels: Vec<Arc<FakeChannel>>) -> (tempfile::TempDir, NotificationFeature) {
    feature_with("", channels)
}

fn feature_with(
    text: &str,
    channels: Vec<Arc<FakeChannel>>,
) -> (tempfile::TempDir, NotificationFeature) {
    let (directory, configuration) = store(text);
    let feature = NotificationFeature::new(
        configuration,
        channels
            .into_iter()
            .map(|channel| channel as Arc<dyn Channel>)
            .collect(),
    )
    .unwrap();
    (directory, feature)
}

#[test]
fn a_service_that_goes_down_and_comes_back_is_announced_twice_and_the_first_up_is_not() {
    let telegram = Arc::new(FakeChannel::ready("telegram"));
    let (_directory, feature) = feature(vec![telegram]);
    let observer = feature.observer();
    observer.changed(&change("unknown", "up"));
    observer.changed(&change("up", "down"));
    observer.changed(&change("down", "up"));
    let outbox = feature.outbox("telegram").unwrap();
    assert_eq!(outbox.waiting(), 2);
    let first = outbox.take().unwrap();
    assert_eq!(first.title, "NAS");
    assert!(
        first.text.contains("up → down") && first.text.contains("connection refused"),
        "{}",
        first.text
    );
}

#[test]
fn an_opted_out_service_and_a_disabled_channel_announce_nothing() {
    let telegram = Arc::new(FakeChannel {
        ready: false,
        ..FakeChannel::ready("telegram")
    });
    let other = Arc::new(FakeChannel::ready("other"));
    let (_directory, feature) = feature(vec![telegram, other]);
    let observer = feature.observer();
    observer.changed(&StatusChange {
        notify: false,
        ..change("up", "down")
    });
    observer.changed(&change("up", "down"));
    assert_eq!(feature.outbox("telegram").unwrap().waiting(), 0);
    assert_eq!(feature.outbox("other").unwrap().waiting(), 1);
}

#[tokio::test]
async fn two_channels_each_deliver_a_failure_through_their_own_queue() {
    let telegram = Arc::new(FakeChannel::ready("telegram"));
    let mail = Arc::new(FakeChannel::ready("mail"));
    let (_directory, feature) = feature(vec![telegram.clone(), mail.clone()]);
    for task in portal_feature::Feature::loops(&feature) {
        tokio::spawn(task);
    }
    feature.observer().changed(&change("up", "down"));
    for _ in 0..50 {
        if !telegram.sent().is_empty() && !mail.sent().is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert_eq!(telegram.sent().len(), 1);
    assert_eq!(mail.sent().len(), 1);
}

#[tokio::test]
async fn a_channel_that_never_answers_holds_up_neither_the_portal_nor_another_channel() {
    let slow = Arc::new(FakeChannel {
        delay: Duration::from_secs(30),
        ..FakeChannel::ready("telegram")
    });
    let fast = Arc::new(FakeChannel::ready("mail"));
    let (_directory, feature) = feature(vec![slow, fast.clone()]);
    for task in portal_feature::Feature::loops(&feature) {
        tokio::spawn(task);
    }
    let observer = feature.observer();
    let started = Instant::now();
    for _ in 0..20 {
        observer.changed(&change("up", "down"));
    }
    assert!(started.elapsed() < Duration::from_millis(100));
    for _ in 0..50 {
        if fast.sent().len() == 20 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert_eq!(fast.sent().len(), 20);
}

#[tokio::test]
async fn a_workflow_notification_waits_for_its_delivery_and_names_a_missing_channel() {
    let telegram = Arc::new(FakeChannel::ready("telegram"));
    let failing = Arc::new(FakeChannel {
        failing: true,
        ..FakeChannel::ready("mail")
    });
    let (_directory, feature) = feature(vec![telegram.clone(), failing]);
    let send = feature.send_notification();
    let deliveries = send
        .run(None, &Notification::new("", "nas is back"))
        .await
        .unwrap();
    assert_eq!(
        deliveries
            .iter()
            .map(|delivery| (delivery.channel.as_str(), delivery.succeeded()))
            .collect::<Vec<_>>(),
        vec![("telegram", true), ("mail", false)]
    );
    assert_eq!(telegram.sent(), vec!["nas is back"]);
    let one = send
        .run(Some("telegram"), &Notification::new("", "only here"))
        .await
        .unwrap();
    assert_eq!(one.len(), 1);
    assert!(
        send.run(Some("sms"), &Notification::new("", "x"))
            .await
            .unwrap_err()
            .contains("sms")
    );
}

#[test]
fn a_channel_with_a_configuration_problem_refuses_to_start_naming_the_key() {
    let (_directory, configuration) = store("");
    let broken = Arc::new(FakeChannel {
        problems: vec![FieldError::new(
            "notifications.telegram.secret",
            "must name the secret holding the bot token",
        )],
        ..FakeChannel::ready("telegram")
    });
    let problem = match NotificationFeature::new(configuration, vec![broken as Arc<dyn Channel>]) {
        Err(problem) => problem,
        Ok(_) => panic!("a broken channel must refuse to start"),
    };
    assert!(
        problem.contains("notifications.telegram.secret"),
        "{problem}"
    );
}

#[tokio::test]
async fn while_the_module_is_off_nothing_is_announced_or_sent() {
    let telegram = Arc::new(FakeChannel::ready("telegram"));
    let (_directory, feature) = feature_with("[modules]\nnotifications = false\n", vec![telegram]);
    feature.observer().changed(&change("up", "down"));
    assert_eq!(feature.outbox("telegram").unwrap().waiting(), 0);
    let refused = feature
        .send_notification()
        .run(None, &Notification::new("", "x"))
        .await
        .unwrap_err();
    assert!(refused.contains("module is off"), "{refused}");
}
