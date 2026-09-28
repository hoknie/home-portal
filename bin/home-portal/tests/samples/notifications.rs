use portal_notification::{
    ChannelResponse, DeliveryResponse, LastErrorResponse, MissingResponse, NotificationsResponse,
    RulesResponse,
};
use serde_json::json;

use crate::check;

fn delivered() -> DeliveryResponse {
    DeliveryResponse {
        channel: "telegram".into(),
        at: "2026-09-28T09:30:00Z".into(),
        delivered: true,
        error: None,
    }
}

#[test]
fn the_notification_samples_match_their_serializers() {
    let listed = NotificationsResponse {
        enabled: true,
        rules: RulesResponse {
            states: vec!["down".into(), "unreadable".into()],
            recovered: true,
        },
        channels: vec![ChannelResponse {
            name: "telegram".into(),
            readiness: "ready".into(),
            missing: None,
            settings: json!({"enabled": true, "secret": "telegram_token", "chat_id": "123456789"}),
            last_delivery: Some(delivered()),
            last_error: Some(LastErrorResponse {
                at: "2026-09-27T22:10:00Z".into(),
                message: "telegram answered 502 Bad Gateway".into(),
            }),
            queued: 0,
            dropped: 0,
        }],
    };
    check("notifications", serde_json::to_value(&listed).unwrap());
    let missing = ChannelResponse {
        name: "telegram".into(),
        readiness: "missing".into(),
        missing: Some(MissingResponse {
            field: "notifications.telegram.secret".into(),
            message: "names telegram_token, which is not set".into(),
        }),
        settings: json!({"enabled": true, "secret": "telegram_token", "chat_id": "123456789"}),
        last_delivery: None,
        last_error: None,
        queued: 3,
        dropped: 0,
    };
    check(
        "notification-channel-missing",
        serde_json::to_value(&missing).unwrap(),
    );
    check(
        "notification-test",
        serde_json::to_value(delivered()).unwrap(),
    );
}
