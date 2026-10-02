use portal_feature::{EventName, FieldError};

use super::{Automation, AutomationsSection, Outcome, RawAutomation, Tail};

fn raw(text: &str) -> RawAutomation {
    let document = format!("[[automations]]\n{text}").parse().unwrap();
    AutomationsSection::read(&document)
        .unwrap()
        .automations
        .remove(0)
}

fn fields(errors: Vec<FieldError>) -> Vec<String> {
    errors.into_iter().map(|error| error.field).collect()
}

const VALID: &str = r#"id = "restart-media"
title = "Restart"
when = { event = "service.status-changed", services = ["jellyfin"], to = ["down"] }
run = { script = "restart.sh", args = ["{{service.id}}"] }
"#;

#[test]
fn the_valid_entry_of_the_specification_decodes() {
    let automation = Automation::decode(&raw(VALID)).unwrap();
    assert_eq!(automation.trigger.event, EventName::ServiceStatusChanged);
    assert_eq!(automation.trigger.filters.services, vec!["jellyfin"]);
    assert_eq!(automation.trigger.filters.states.to, vec!["down"]);
    assert!(!automation.trigger.filters.states.from_unknown);
    assert_eq!(automation.run.timeout_seconds, 60);
    assert!(automation.enabled);
    assert_eq!(automation.cooldown_seconds, 0);
}

#[test]
fn a_filter_of_another_event_is_refused_by_its_key() {
    let errors = Automation::decode(&raw(r#"id = "a"
title = "A"
when = { event = "user.signed-in", to = ["down"] }
run = { script = "a.sh" }
"#))
    .unwrap_err();
    assert_eq!(fields(errors.clone()), vec!["when.to"]);
    assert!(errors[0].message.contains("users, environments"));
}

#[test]
fn a_schedule_needs_a_cron_expression() {
    let errors = Automation::decode(&raw(
        "id = \"a\"\ntitle = \"A\"\nwhen = { event = \"schedule\" }\nrun = { script = \"a.sh\" }\n",
    ))
    .unwrap_err();
    assert_eq!(fields(errors), vec!["when.cron"]);
}

#[test]
fn a_timeout_outside_one_to_an_hour_is_refused() {
    for timeout in [0, 3601] {
        let errors = Automation::decode(&raw(&format!(
            "id = \"a\"\ntitle = \"A\"\nwhen = {{ event = \"portal.started\" }}\nrun = {{ script = \"a.sh\", timeout_seconds = {timeout} }}\n"
        )))
        .unwrap_err();
        assert_eq!(fields(errors), vec!["run.timeout_seconds"], "{timeout}");
    }
}

#[test]
fn from_unknown_in_the_list_needs_the_flag() {
    let text = |flag: &str| {
        format!(
            "id = \"a\"\ntitle = \"A\"\nwhen = {{ event = \"service.status-changed\", from = [\"unknown\"]{flag} }}\nrun = {{ script = \"a.sh\" }}\n"
        )
    };
    let errors = Automation::decode(&raw(&text(""))).unwrap_err();
    assert_eq!(fields(errors), vec!["when.from"]);
    assert!(Automation::decode(&raw(&text(", from_unknown = true"))).is_ok());
}

#[test]
fn a_state_outside_the_vocabulary_and_an_unknown_event_are_refused() {
    let errors = Automation::decode(&raw(
        "id = \"a\"\ntitle = \"A\"\nwhen = { event = \"service.status-changed\", to = [\"broken\"] }\nrun = { script = \"a.sh\" }\n",
    ))
    .unwrap_err();
    assert_eq!(fields(errors), vec!["when.to"]);
    let errors = Automation::decode(&raw(
        "id = \"a\"\ntitle = \"A\"\nwhen = { event = \"service.exploded\" }\nrun = { script = \"a.sh\" }\n",
    ))
    .unwrap_err();
    assert_eq!(fields(errors), vec!["when.event"]);
}

#[test]
fn an_invocation_carries_the_fields_as_arguments_variables_and_input() {
    use std::path::PathBuf;

    use portal_feature::PortalEvent;
    use time::OffsetDateTime;

    use super::{Invocation, Pending};

    let automation = Automation::decode(&raw(r#"id = "restart-media"
title = "Restart"
when = { event = "service.status-changed" }
run = { script = "restart.sh", args = ["--", "{{service.id}}", "{{run.manual}}"] }
"#))
    .unwrap();
    let event = PortalEvent::of(
        EventName::ServiceStatusChanged,
        OffsetDateTime::UNIX_EPOCH,
        &[("service.id", "nas")],
    );
    let pending = Pending {
        run_id: 7,
        automation,
        event,
        by: None,
        origin: Vec::new(),
    };
    let invocation = Invocation::for_run(
        &pending,
        PathBuf::from("/s/restart.sh"),
        PathBuf::from("/s"),
    );
    assert_eq!(invocation.arguments, vec!["--", "nas", "false"]);
    let variable = |name: &str| {
        invocation
            .environment
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    };
    assert_eq!(variable("PORTAL_SERVICE_ID"), Some("nas"));
    assert_eq!(
        variable("PORTAL_EVENT_NAME"),
        Some("service.status-changed")
    );
    assert_eq!(variable("PORTAL_AUTOMATION_ID"), Some("restart-media"));
    assert_eq!(variable("PORTAL_RUN_ID"), Some("7"));
    let input: serde_json::Value = serde_json::from_str(&invocation.input).unwrap();
    assert_eq!(input["service.id"], "nas");
    assert_eq!(input["status.error"], "");
    assert_eq!(invocation.timeout.as_secs(), 60);
}

#[test]
fn a_cut_inside_a_cyrillic_character_starts_at_the_next_character() {
    let mut tail = Tail::default();
    let body = "я".repeat(100 * 1024);
    tail.push(format!("x{body}").as_bytes());
    tail.push("готово\n".as_bytes());
    let text = tail.text();
    assert!(!text.starts_with('\u{FFFD}'));
    assert!(text.ends_with("готово\n"));
    assert_eq!(tail.total, 1 + 200 * 1024 + "готово\n".len() as u64);
    assert!(tail.truncated());
    assert!(Tail::KEPT - text.len() <= 3);
}

#[test]
fn a_restored_tail_starts_on_a_character_boundary() {
    let stored = format!("x{}", "я".repeat(40 * 1024));
    let tail = Tail::restored(stored.as_bytes(), 1_000_000);
    assert!(!tail.text().starts_with('\u{FFFD}'));
    assert_eq!(tail.total, 1_000_000);
}

#[test]
fn a_short_output_is_kept_whole() {
    let mut tail = Tail::default();
    tail.push("привет\n".as_bytes());
    assert_eq!(tail.text(), "привет\n");
    assert!(!tail.truncated());
}

#[test]
fn every_outcome_reads_back_by_its_name_and_stopped_is_one_of_them() {
    for outcome in Outcome::ALL {
        assert_eq!(Outcome::of(outcome.name()), Some(outcome));
    }
    assert_eq!(Outcome::of("stopped"), Some(Outcome::Stopped));
}

#[test]
fn every_step_kind_has_a_catalogue_entry_and_every_entry_a_kind() {
    use super::{
        Condition, HttpStep, KINDS, LoopMode, Operator, RunSettings, SetValue, StepKind, kind_named,
    };
    let condition = Condition::Compare {
        left: String::new(),
        operator: Operator::IsEmpty,
        right: None,
    };
    let every = [
        StepKind::If {
            condition: condition.clone(),
            then: Vec::new(),
            otherwise: Vec::new(),
        },
        StepKind::Loop {
            mode: LoopMode::Repeat(crate::types::NumberSetting::Fixed(1)),
            max_iterations: crate::types::NumberSetting::Fixed(1),
            body: Vec::new(),
        },
        StepKind::Parallel {
            branches: Vec::new(),
        },
        StepKind::Call {
            workflow: String::new(),
            inputs: Vec::new(),
        },
        StepKind::Stop {
            succeeded: true,
            reason: None,
        },
        StepKind::Set {
            variable: String::new(),
            value: SetValue::Text(String::new()),
        },
        StepKind::Wait {
            seconds: crate::types::NumberSetting::Fixed(1),
        },
        StepKind::Transform {
            input: String::new(),
            operations: Vec::new(),
        },
        StepKind::Http(HttpStep {
            method: "GET".into(),
            url: String::new(),
            headers: Vec::new(),
            body: None,
            timeout_seconds: crate::types::NumberSetting::Fixed(1),
            fail_on_error: true,
            response_sample: None,
        }),
        StepKind::Script {
            run: RunSettings {
                script: String::new(),
                args: Vec::new(),
                timeout_seconds: 1,
            },
            env: Vec::new(),
            stdin: None,
            timeout: crate::types::NumberSetting::Fixed(1),
            fail_on_error: true,
        },
        StepKind::Notify {
            channel: None,
            title: None,
            text: String::new(),
        },
        StepKind::Probe {
            service: String::new(),
        },
        StepKind::Status {
            service: String::new(),
        },
        StepKind::Nothing,
        StepKind::Break,
        StepKind::Continue,
        StepKind::Log {
            message: String::new(),
            level: crate::types::LogLevel::Info,
        },
        StepKind::Automation {
            automation: String::new(),
            fields: Vec::new(),
            wait: false,
        },
    ];
    for kind in &every {
        assert!(kind_named(kind.name()).is_some(), "{}", kind.name());
    }
    let names: Vec<&str> = every.iter().map(StepKind::name).collect();
    for kind in KINDS {
        assert!(names.contains(&kind.name), "{}", kind.name);
    }
}

#[test]
fn every_exclusive_group_names_fields_of_its_kind() {
    use crate::types::{KINDS, kind_named};
    for kind in KINDS {
        for group in kind.exclusive {
            assert!(group.len() > 1, "{}", kind.name);
            for field in *group {
                assert!(
                    kind.fields.iter().any(|candidate| candidate.name == *field),
                    "{}.{field}",
                    kind.name
                );
            }
        }
    }
    let exclusive = |name: &str| kind_named(name).map(|kind| kind.exclusive.len());
    assert_eq!(exclusive("loop"), Some(1));
    assert_eq!(exclusive("set"), Some(1));
}

#[test]
fn an_old_trace_entry_loads_without_a_log_and_a_new_one_keeps_it() {
    let old = r#"{"path":"steps[0]","step":"ping","label":"ping","kind":"http","iteration":null,"outcome":"failed","started_at":"2026-01-01T00:00:00Z","duration_milliseconds":3,"detail":"500","output":null}"#;
    let entry = serde_json::from_str::<super::stored::StoredTraceEntry>(old)
        .unwrap()
        .into_entry()
        .unwrap();
    assert!(entry.log.is_empty());
    assert_eq!((entry.item, entry.level), (None, None));
    let mut log = super::StepLog::default();
    log.push_value("{{loop.item}}", "\"Media\"");
    log.push_line("no service Media");
    let written = super::stored::StoredTraceEntry::of(&super::TraceEntry {
        log,
        item: Some("\"Media\"".into()),
        level: Some(super::LogLevel::Warning),
        ..entry
    });
    let line = serde_json::to_string(&written).unwrap();
    let back = serde_json::from_str::<super::stored::StoredTraceEntry>(&line)
        .unwrap()
        .into_entry()
        .unwrap();
    assert_eq!(back.log.values[0].value, "\"Media\"");
    assert_eq!(back.log.lines, vec!["no service Media"]);
    assert_eq!(back.level, Some(super::LogLevel::Warning));
    assert!(
        !serde_json::to_string(&super::stored::StoredTraceEntry::of(&super::TraceEntry {
            log: super::StepLog::default(),
            item: None,
            level: None,
            ..back
        }))
        .unwrap()
        .contains("values")
    );
}

#[test]
fn a_step_log_keeps_twenty_values_and_lines_of_bounded_length_and_counts_the_rest() {
    let mut log = super::StepLog::default();
    for index in 0..25 {
        log.push_value(&format!("{{{{vars.v{index}}}}}"), &"x".repeat(400));
        log.push_line(format!("line {index}"));
    }
    assert_eq!((log.values.len(), log.values_dropped), (20, 5));
    assert_eq!((log.lines.len(), log.lines_dropped), (20, 5));
    assert!(log.values[0].value.chars().count() <= super::StepLog::LONGEST + 1);
    let dropped = log.dropped();
    assert!(dropped.is_empty());
    assert_eq!((dropped.values_dropped, dropped.lines_dropped), (25, 25));
}

#[test]
fn a_record_from_before_separate_streams_keeps_its_output_and_new_streams_survive_the_journal() {
    let old = r#"{"path":"steps[0]","step":"run","label":"run","kind":"script","iteration":null,"outcome":"failed","started_at":"2026-01-01T00:00:00Z","duration_milliseconds":3,"detail":"restart.sh exited 1","output":"progress\ncontainer not found\n"}"#;
    let entry = serde_json::from_str::<super::stored::StoredTraceEntry>(old)
        .unwrap()
        .into_entry()
        .unwrap();
    assert_eq!(
        entry.output.as_deref(),
        Some("progress\ncontainer not found\n")
    );
    assert_eq!(entry.streams, None);
    let mut stdout = super::Tail::default();
    stdout.push(&[b'x'; 20 * 1024]);
    let mut stderr = super::Tail::default();
    stderr.push(b"container not found\n");
    let streams = super::Streams {
        stdout: stdout.last(super::Streams::KEPT_PER_STREAM),
        stderr,
        command: vec!["restart.sh".into(), "jellyfin".into()],
        budget_reached: true,
    };
    let line = serde_json::to_string(&super::stored::StoredTraceEntry::of(&super::TraceEntry {
        output: None,
        streams: Some(streams.clone()),
        ..entry
    }))
    .unwrap();
    let back = serde_json::from_str::<super::stored::StoredTraceEntry>(&line)
        .unwrap()
        .into_entry()
        .unwrap();
    assert_eq!(back.streams, Some(streams));
    assert!(back.streams.unwrap().stdout.truncated());
}
