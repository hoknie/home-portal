use portal_feature::{EventName, FieldError};

use super::{Automation, AutomationsSection, RawAutomation};

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
