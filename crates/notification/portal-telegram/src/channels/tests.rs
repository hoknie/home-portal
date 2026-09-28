use std::collections::BTreeMap;
use std::time::Duration;

use portal_feature::{Channel, ChannelReadiness, Notification, SecretSource};
use toml_edit::DocumentMut;

use super::TelegramChannel;
use crate::fakes::BotService;

struct Secrets(BTreeMap<String, String>);

impl SecretSource for Secrets {
    fn reveal(&self, name: &str) -> Option<String> {
        self.0.get(name).cloned()
    }
}

fn secrets() -> Secrets {
    Secrets(BTreeMap::from([(
        "telegram_token".to_string(),
        "abc".to_string(),
    )]))
}

const ENABLED: &str =
    "[notifications.telegram]\nenabled = true\nsecret = \"telegram_token\"\nchat_id = \"42\"\n";

fn document(text: &str) -> DocumentMut {
    text.parse().unwrap()
}

fn channel() -> TelegramChannel {
    TelegramChannel::new("http://127.0.0.1:9").unwrap()
}

#[test]
fn enabling_telegram_without_a_token_or_a_chat_names_the_key() {
    let fields: Vec<String> = channel()
        .problems(
            &document("[notifications.telegram]\nenabled = true\n"),
            &secrets(),
        )
        .into_iter()
        .map(|error| error.field)
        .collect();
    assert_eq!(
        fields,
        vec![
            "notifications.telegram.secret",
            "notifications.telegram.chat_id"
        ]
    );
    let errors = channel().problems(
        &document(
            "[notifications.telegram]\nenabled = true\nsecret = \"nope\"\nchat_id = \"42\"\n",
        ),
        &secrets(),
    );
    assert!(errors[0].message.contains("nope"), "{}", errors[0].message);
    assert!(!errors[0].message.contains("abc"));
    assert!(
        channel()
            .problems(
                &document("[notifications.telegram]\nchat_id = \"42\"\n"),
                &secrets()
            )
            .is_empty()
    );
}

#[test]
fn readiness_says_disabled_missing_or_ready() {
    assert_eq!(
        channel().readiness(&document(""), &secrets()),
        ChannelReadiness::Disabled
    );
    let missing = channel().readiness(
        &document("[notifications.telegram]\nenabled = true\nsecret = \"telegram_token\"\n"),
        &secrets(),
    );
    assert!(
        matches!(missing, ChannelReadiness::Missing { ref field, .. } if field == "notifications.telegram.chat_id")
    );
    assert_eq!(
        channel().readiness(&document(ENABLED), &secrets()),
        ChannelReadiness::Ready
    );
}

#[test]
fn the_settings_view_names_the_secret_but_never_its_value() {
    let view = channel().settings(&document(ENABLED));
    assert_eq!(view["secret"], "telegram_token");
    assert_eq!(view["chat_id"], "42");
    assert!(!view.to_string().contains("abc"));
}

#[test]
fn applying_settings_keeps_comments_and_refuses_an_enabled_channel_without_a_chat() {
    let mut document =
        document("[notifications.telegram]\n# the family chat\nchat_id = \"1\" # old\n");
    let table = document["notifications"]["telegram"]
        .as_table_mut()
        .unwrap();
    channel()
        .apply(
            table,
            &serde_json::json!({"enabled": true, "secret": "telegram_token", "chat_id": "42"}),
        )
        .unwrap();
    let text = document.to_string();
    assert!(text.contains("# the family chat"), "{text}");
    assert!(text.contains("chat_id = \"42\""), "{text}");
    assert!(text.contains("enabled = true"), "{text}");
    let table = document["notifications"]["telegram"]
        .as_table_mut()
        .unwrap();
    let errors = channel()
        .apply(
            table,
            &serde_json::json!({"enabled": true, "secret": "telegram_token"}),
        )
        .unwrap_err();
    assert_eq!(errors[0].field, "chat_id");
}

#[tokio::test]
async fn a_notification_reaches_telegram_with_the_token_and_the_chat() {
    let telegram = BotService::start(200, Duration::ZERO).await;
    let channel = TelegramChannel::new(&telegram.endpoint()).unwrap();
    channel
        .deliver(
            &Notification::new("NAS", "up → down"),
            &document(ENABLED),
            &secrets(),
        )
        .await
        .unwrap();
    let request = telegram.requests().first().cloned().unwrap_or_default();
    assert!(request.contains("/botabc/sendMessage"), "{request}");
    assert!(request.contains("\"chat_id\":\"42\""), "{request}");
    assert!(request.contains("NAS"), "{request}");
}

#[tokio::test]
async fn delivering_without_telegram_configured_is_refused() {
    let refused = channel()
        .deliver(&Notification::new("", "hello"), &document(""), &secrets())
        .await
        .unwrap_err();
    assert!(refused.contains("not configured"), "{refused}");
}
