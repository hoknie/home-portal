use std::collections::BTreeMap;

use portal_automations::{
    AutomationResponse, AutomationsResponse, CatalogueResponse, Choice, Directory, MarksResponse,
    OutcomeResponse, OutputResponse, QueuedResponse, Refusal, RefusalCode, RunResponse,
    RunSettingsResponse, RunsResponse, ScheduleResponse, ScriptEntry, ScriptResponse,
    ScriptsResponse, StatesResponse, WhenResponse, WorkflowCallResponse,
};
use portal_model::ScriptHeader;

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

pub fn run(id: &str, automation: &str, event: &str, outcome: OutcomeResponse) -> RunResponse {
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
        workflow: None,
        trace: None,
        steps_version: None,
    }
}

pub fn outcome(result: &str, exit_code: Option<i32>, reason: Option<&str>) -> OutcomeResponse {
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
    let mut revived = run(
        "0",
        "nas-down",
        "service.status-changed",
        outcome("succeeded", None, Some("nas is back")),
    );
    revived.arguments = Vec::new();
    revived.workflow = Some("revive".into());
    revived.trace = Some(crate::workflows::traced(&[
        (
            "steps[0]",
            "first_probe",
            "probe",
            None,
            "succeeded",
            "nas: down",
        ),
        ("steps[1]", "down", "if", None, "succeeded", "then"),
        (
            "steps[1].then[0]",
            "retry",
            "loop",
            None,
            "succeeded",
            "1 iterations",
        ),
        (
            "steps[1].then[0].body[0]",
            "restart",
            "http",
            Some(0),
            "succeeded",
            "POST http://192.168.1.10:9000/restart/nas → 202",
        ),
        (
            "steps[1].then[0].body[1]",
            "settle",
            "wait",
            Some(0),
            "running",
            "",
        ),
    ]));
    vec![
        run("5", "backup", "manual", running),
        run("4", "backup", "manual", stopped),
        run("3", "backup", "schedule", failed),
        run("2", "restart-media", "service.status-changed", skipped),
        run("1", "restart-media", "service.status-changed", succeeded),
        revived,
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
                run: Some(RunSettingsResponse {
                    script: "restart.sh".into(),
                    args: vec!["--".into(), "{{service.id}}".into()],
                    timeout_seconds: 120,
                }),
                workflow: None,
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
                run: Some(RunSettingsResponse {
                    script: "backup/nightly.sh".into(),
                    args: Vec::new(),
                    timeout_seconds: 60,
                }),
                workflow: None,
                last_run: Some(history[1].clone()),
                active_run: Some(history[0].clone()),
            },
            AutomationResponse {
                id: "motion-alarm".into(),
                title: "Alarm on motion".into(),
                marks: MarksResponse {
                    enabled: true,
                    tags: vec!["camera".into()],
                },
                cooldown_seconds: 0,
                when: WhenResponse {
                    event: "webhook.received".into(),
                    cron: None,
                    services: Vec::new(),
                    states: StatesResponse {
                        from: Vec::new(),
                        to: Vec::new(),
                        from_unknown: false,
                    },
                    users: Vec::new(),
                    environments: Vec::new(),
                    webhooks: vec!["0b9e8c2a-1d3f-4a5b-8c7d-9e0f1a2b3c4d".into()],
                },
                run: None,
                workflow: Some(WorkflowCallResponse {
                    id: "revive".into(),
                    inputs: BTreeMap::from([(
                        "service".to_string(),
                        serde_json::Value::String("{{webhook.camera}}".into()),
                    )]),
                }),
                last_run: None,
                active_run: None,
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
            &super::webhooks(),
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
            editing: false,
            scripts: vec![
                ScriptResponse::of(ScriptEntry {
                    path: "backup/nightly.sh".into(),
                    problem: None,
                    header: ScriptHeader::default(),
                }),
                ScriptResponse::of(ScriptEntry {
                    path: "media/restart.sh".into(),
                    problem: None,
                    header: ScriptHeader::parse(
                        "#!/bin/sh\n# @description Restart a service's container\n# @arg service <text> Service id\n# @arg --retries <number=3> Tries before giving up\n# @arg --force Skip the health check\n# @arg mode <fast|full=fast> How deep to check\n",
                    ),
                }),
                ScriptResponse::of(ScriptEntry {
                    path: "open.sh".into(),
                    problem: Some(Refusal {
                        code: RefusalCode::Writable,
                        path: "/srv/home-portal/scripts/open.sh".into(),
                        message: "open.sh can be written by group or others".into(),
                    }),
                    header: ScriptHeader::parse("# @arg --keep <weird> Days\n"),
                }),
                ScriptResponse::of(ScriptEntry {
                    path: "restart.sh".into(),
                    problem: None,
                    header: ScriptHeader::default(),
                }),
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
