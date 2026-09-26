use std::collections::BTreeMap;

use portal_automations::{
    AcceptedResponse, AutomationResponse, AutomationsResponse, CatalogueResponse, Choice,
    CreatedWebhookResponse, Directory, MarksResponse, OutcomeResponse, OutputResponse,
    QueuedResponse, RawMarks, RawRun, RawWebhook, ReceptionResponse, RunResponse,
    RunSettingsResponse, RunsResponse, ScheduleResponse, ScriptResponse, ScriptsResponse,
    StatesResponse, TokenResponse, Webhook, WebhookResponse, WebhooksResponse, WhenResponse,
};

use crate::check;

struct Home;

impl Directory for Home {
    fn services(&self) -> Vec<Choice> {
        vec![
            Choice {
                id: "jellyfin".into(),
                name: "Jellyfin".into(),
            },
            Choice {
                id: "nas".into(),
                name: "NAS".into(),
            },
        ]
    }

    fn users(&self) -> Vec<String> {
        vec!["admin".into()]
    }

    fn environments(&self) -> Vec<String> {
        vec!["home".into(), "vpn".into(), "internet".into()]
    }
}

fn output(tail: &str, bytes: u64) -> OutputResponse {
    OutputResponse {
        tail: tail.into(),
        bytes,
        truncated: bytes > tail.len() as u64,
    }
}

fn run(id: &str, automation: &str, event: &str, outcome: OutcomeResponse) -> RunResponse {
    let fields: BTreeMap<String, String> = [
        ("event.name", event),
        ("event.at", "2026-09-25T03:00:00Z"),
        ("automation.id", automation),
        ("run.id", id),
        ("run.manual", "false"),
        ("run.by", ""),
    ]
    .into_iter()
    .map(|(key, value)| (key.to_string(), value.to_string()))
    .collect();
    RunResponse {
        id: id.into(),
        automation: automation.into(),
        event: event.into(),
        fields,
        arguments: vec!["--".into(), "jellyfin".into()],
        started_at: "2026-09-25T03:00:00Z".into(),
        outcome,
    }
}

fn outcome(result: &str, exit_code: Option<i32>, reason: Option<&str>) -> OutcomeResponse {
    OutcomeResponse {
        result: result.into(),
        exit_code,
        reason: reason.map(str::to_string),
        duration_milliseconds: 1840,
        count: 1,
        last_at: "2026-09-25T03:00:00Z".into(),
        stdout: output("", 0),
        stderr: output("", 0),
    }
}

fn runs() -> Vec<RunResponse> {
    let mut failed = outcome("failed", Some(1), None);
    failed.stderr = output("disk full\n", 10);
    let mut succeeded = outcome("succeeded", Some(0), None);
    succeeded.stdout = output("restarted jellyfin\n", 70_000);
    let mut skipped = outcome("skipped", None, Some("cooldown"));
    skipped.duration_milliseconds = 0;
    skipped.count = 9;
    skipped.last_at = "2026-09-25T03:04:00Z".into();
    let mut running = outcome("running", None, None);
    running.duration_milliseconds = 12_000;
    running.stdout = output("copying\n", 8);
    running.stderr = output("  % Total\r 10  1.2M\r 55  6.6M", 28);
    let mut stopped = outcome("stopped", None, Some("stopped by admin"));
    stopped.stdout = output("syncing\n", 8);
    vec![
        run("5", "backup", "manual", running),
        run("4", "backup", "manual", stopped),
        run("3", "backup", "schedule", failed),
        run("2", "restart-media", "service.status-changed", skipped),
        run("1", "restart-media", "service.status-changed", succeeded),
    ]
}

#[test]
fn the_automation_samples_match_their_serializers() {
    let history = runs();
    let automations = AutomationsResponse {
        automations: vec![
            AutomationResponse {
                id: "restart-media".into(),
                title: "Restart Jellyfin when it goes down".into(),
                marks: MarksResponse {
                    enabled: true,
                    tags: vec!["media".into(), "night".into()],
                },
                cooldown_seconds: 300,
                when: WhenResponse {
                    event: "service.status-changed".into(),
                    cron: None,
                    services: vec!["jellyfin".into()],
                    states: StatesResponse {
                        from: Vec::new(),
                        to: vec!["down".into(), "unreadable".into()],
                        from_unknown: false,
                    },
                    users: Vec::new(),
                    environments: Vec::new(),
                    webhooks: Vec::new(),
                },
                run: RunSettingsResponse {
                    script: "restart.sh".into(),
                    args: vec!["--".into(), "{{service.id}}".into()],
                    timeout_seconds: 120,
                },
                last_run: Some(history[3].clone()),
                active_run: None,
            },
            AutomationResponse {
                id: "backup".into(),
                title: "Nightly backup".into(),
                marks: MarksResponse {
                    enabled: false,
                    tags: vec!["backup".into()],
                },
                cooldown_seconds: 0,
                when: WhenResponse {
                    event: "schedule".into(),
                    cron: Some("0 3 * * *".into()),
                    services: Vec::new(),
                    states: StatesResponse {
                        from: Vec::new(),
                        to: Vec::new(),
                        from_unknown: false,
                    },
                    users: Vec::new(),
                    environments: Vec::new(),
                    webhooks: Vec::new(),
                },
                run: RunSettingsResponse {
                    script: "backup/nightly.sh".into(),
                    args: Vec::new(),
                    timeout_seconds: 60,
                },
                last_run: Some(history[1].clone()),
                active_run: Some(history[0].clone()),
            },
        ],
    };
    check("automations", serde_json::to_value(&automations).unwrap());
    check(
        "automation-runs",
        serde_json::to_value(RunsResponse { runs: history }).unwrap(),
    );
    check(
        "automation-catalogue",
        serde_json::to_value(CatalogueResponse::of(
            &Home,
            &webhooks(),
            vec![
                "backup".into(),
                "camera".into(),
                "ci".into(),
                "media".into(),
                "night".into(),
            ],
        ))
        .unwrap(),
    );
    check(
        "automation-scripts",
        serde_json::to_value(ScriptsResponse {
            directory: "/srv/home-portal/scripts".into(),
            exists: true,
            user_id: 501,
            scripts: vec![
                ScriptResponse {
                    path: "backup/nightly.sh".into(),
                    runnable: true,
                    problem: None,
                    code: None,
                    concerns: None,
                },
                ScriptResponse {
                    path: "open.sh".into(),
                    runnable: false,
                    problem: Some("open.sh can be written by group or others".into()),
                    code: Some("writable".into()),
                    concerns: Some("/srv/home-portal/scripts/open.sh".into()),
                },
                ScriptResponse {
                    path: "restart.sh".into(),
                    runnable: true,
                    problem: None,
                    code: None,
                    concerns: None,
                },
            ],
        })
        .unwrap(),
    );
    check(
        "automation-schedule",
        serde_json::to_value(ScheduleResponse {
            timezone: "Europe/Berlin".into(),
            times: vec![
                "2026-09-28T03:00:00+02:00".into(),
                "2026-09-29T03:00:00+02:00".into(),
                "2026-09-30T03:00:00+02:00".into(),
                "2026-10-01T03:00:00+02:00".into(),
                "2026-10-02T03:00:00+02:00".into(),
            ],
        })
        .unwrap(),
    );
    check(
        "automation-queued",
        serde_json::to_value(QueuedResponse {
            run_id: "42".into(),
        })
        .unwrap(),
    );
}

fn webhooks() -> Vec<Webhook> {
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
        token_sha256: None,
    };
    [deploy, motion]
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
