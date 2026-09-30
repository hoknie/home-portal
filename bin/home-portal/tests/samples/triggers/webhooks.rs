use std::collections::BTreeMap;

use portal_automations::{
    AcceptedResponse, CreatedWebhookResponse, InputValue, RawMarks, RawRun, RawWebhook,
    ReceptionResponse, TokenResponse, Webhook, WebhookResponse, WebhooksResponse,
};

use crate::check;

pub fn webhooks() -> Vec<Webhook> {
    let deploy = RawWebhook {
        id: "7d3f2a4e-5b1c-4e8f-9a2d-6c0b1e3f4a5d".into(),
        title: "Deploy from CI".into(),
        marks: RawMarks {
            enabled: None,
            tags: vec!["ci".into(), "media".into()],
        },
        variables: vec!["branch".into(), "commit".into()],
        action: "script".into(),
        run: Some(RawRun {
            script: "deploy.sh".into(),
            args: vec!["--".into(), "{{webhook.branch}}".into()],
            timeout_seconds: Some(300),
        }),
        workflow: None,
        inputs: None,
        token_sha256: Some("0".repeat(64)),
    };
    let motion = RawWebhook {
        id: "0b9e8c2a-1d3f-4a5b-8c7d-9e0f1a2b3c4d".into(),
        title: "Motion at the door".into(),
        marks: RawMarks {
            enabled: None,
            tags: vec!["camera".into()],
        },
        variables: vec!["camera".into()],
        action: "event".into(),
        run: None,
        workflow: None,
        inputs: None,
        token_sha256: None,
    };
    let release = RawWebhook {
        id: "3c1d9e7f-2a4b-4c6d-8e0f-1a2b3c4d5e6f".into(),
        title: "Release from GitHub".into(),
        marks: RawMarks {
            enabled: None,
            tags: vec!["ci".into()],
        },
        variables: vec!["service".into()],
        action: "script".into(),
        run: None,
        workflow: Some("revive".into()),
        inputs: Some(BTreeMap::from([(
            "service".to_string(),
            InputValue::Template("{{webhook.service}}".into()),
        )])),
        token_sha256: None,
    };
    [deploy, motion, release]
        .iter()
        .map(|raw| Webhook::decode(raw).unwrap())
        .collect()
}

#[test]
fn the_webhook_samples_match_their_serializers() {
    let all = webhooks();
    let listed = WebhooksResponse {
        webhooks: vec![
            WebhookResponse::of(&all[0], None),
            WebhookResponse::of(&all[1], None),
            WebhookResponse::of(&all[2], None),
        ],
    };
    let mut listed = serde_json::to_value(listed).unwrap();
    listed["webhooks"][0]["last_received"] = serde_json::to_value(ReceptionResponse {
        at: "2026-09-25T10:00:00Z".into(),
        status: 202,
    })
    .unwrap();
    check("webhooks", listed);
    check(
        "webhook-created",
        serde_json::to_value(CreatedWebhookResponse {
            webhook: WebhookResponse::of(&all[1], None),
            token: Some("f".repeat(64)),
        })
        .unwrap(),
    );
    check(
        "webhook-token",
        serde_json::to_value(TokenResponse {
            token: "e".repeat(64),
        })
        .unwrap(),
    );
    check(
        "webhook-accepted",
        serde_json::to_value(AcceptedResponse {
            accepted: true,
            run_id: Some("42".into()),
        })
        .unwrap(),
    );
}
