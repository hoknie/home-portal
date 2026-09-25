use portal_feature::FieldError;
use toml_edit::DocumentMut;

use crate::services::validate_automations;

fn errors(text: &str) -> Vec<FieldError> {
    validate_automations(&text.parse::<DocumentMut>().unwrap())
}

fn fields(text: &str) -> Vec<String> {
    errors(text).into_iter().map(|error| error.field).collect()
}

fn entry(id: &str, when: &str, run: &str) -> String {
    format!("[[automations]]\nid = \"{id}\"\ntitle = \"T\"\nwhen = {when}\nrun = {run}\n\n")
}

const STARTED: &str = "{ event = \"portal.started\" }";
const SCRIPT: &str = "{ script = \"a.sh\" }";

#[test]
fn no_automations_is_valid() {
    assert!(errors("").is_empty());
}

#[test]
fn a_duplicate_id_is_refused_on_the_second_entry() {
    let text = entry("a", STARTED, SCRIPT) + &entry("a", STARTED, SCRIPT);
    assert_eq!(fields(&text), vec!["automations[1].id"]);
}

#[test]
fn an_id_with_capitals_or_spaces_is_refused() {
    assert_eq!(
        fields(&entry("Bad Id", STARTED, SCRIPT)),
        vec!["automations[0].id"]
    );
}

#[test]
fn an_unknown_event_is_refused_by_its_path() {
    assert_eq!(
        fields(&entry("a", "{ event = \"nope\" }", SCRIPT)),
        vec!["automations[0].when.event"]
    );
}

#[test]
fn a_foreign_filter_is_refused_by_its_path() {
    assert_eq!(
        fields(&entry(
            "a",
            "{ event = \"user.signed-in\", to = [\"down\"] }",
            SCRIPT
        )),
        vec!["automations[0].when.to"]
    );
}

#[test]
fn a_state_outside_the_vocabulary_is_refused_by_its_path() {
    assert_eq!(
        fields(&entry(
            "a",
            "{ event = \"service.status-changed\", from = [\"sad\"] }",
            SCRIPT
        )),
        vec!["automations[0].when.from"]
    );
}

#[test]
fn a_malformed_cron_is_refused_with_its_range() {
    let found = errors(&entry(
        "a",
        "{ event = \"schedule\", cron = \"61 * * * *\" }",
        SCRIPT,
    ));
    assert_eq!(found[0].field, "automations[0].when.cron");
    assert!(found[0].message.contains("minute must be 0 to 59"));
}

#[test]
fn a_script_path_that_climbs_out_is_refused() {
    for script in ["../home-portal.toml", "/bin/sh", ""] {
        assert_eq!(
            fields(&entry(
                "a",
                STARTED,
                &format!("{{ script = \"{script}\" }}")
            )),
            vec!["automations[0].run.script"],
            "{script}"
        );
    }
}

#[test]
fn an_unknown_placeholder_is_refused_and_lists_the_fields_of_the_event() {
    let found = errors(&entry(
        "a",
        STARTED,
        "{ script = \"a.sh\", args = [\"--\", \"{{service.id}}\"] }",
    ));
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].field, "automations[0].run.args[1]");
    assert!(found[0].message.contains("portal.address, portal.version"));
}

#[test]
fn an_unknown_time_zone_is_refused() {
    let text = format!(
        "[automation_settings]\ntimezone = \"Mars/Olympus\"\n\n{}",
        entry("a", STARTED, SCRIPT)
    );
    assert_eq!(fields(&text), vec!["automation_settings.timezone"]);
}

#[test]
fn a_filter_naming_a_service_that_does_not_exist_is_accepted() {
    assert!(
        errors(&entry(
            "a",
            "{ event = \"service.status-changed\", services = [\"nas\"] }",
            SCRIPT
        ))
        .is_empty()
    );
}

#[test]
fn an_id_the_interface_uses_for_its_own_pages_is_refused() {
    for reserved in ["runs", "catalogue", "scripts", "schedule"] {
        let found = errors(&entry(reserved, STARTED, SCRIPT));
        assert_eq!(found[0].field, "automations[0].id", "{reserved}");
        assert!(found[0].message.contains("reserved"));
    }
}

#[test]
fn a_schedule_that_never_fires_is_refused_on_save() {
    let found = errors(&entry(
        "a",
        "{ event = \"schedule\", cron = \"0 0 30 2 *\" }",
        SCRIPT,
    ));
    assert_eq!(found[0].field, "automations[0].when.cron");
    assert!(found[0].message.starts_with("never fires"));
}

#[test]
fn a_script_too_deep_or_hidden_is_refused_on_save() {
    for script in ["a/b/c.sh", ".cache/tool.sh", ".hidden.sh"] {
        assert_eq!(
            fields(&entry(
                "a",
                STARTED,
                &format!("{{ script = \"{script}\" }}")
            )),
            vec!["automations[0].run.script"],
            "{script}"
        );
    }
}

const WEBHOOK: &str = "[[webhooks]]\nid = \"7d3f2a4e-5b1c-4e8f-9a2d-6c0b1e3f4a5d\"\ntitle = \"Deploy\"\nvariables = [\"branch\"]\naction = \"event\"\n\n";

#[test]
fn a_webhook_placeholder_needs_a_chosen_webhook_that_declares_it() {
    let good = format!(
        "{WEBHOOK}{}",
        entry(
            "a",
            "{ event = \"webhook.received\" }",
            "{ script = \"a.sh\", args = [\"{{webhook.branch}}\", \"{{webhook.title}}\"] }"
        )
    );
    assert!(errors(&good).is_empty(), "{:?}", errors(&good));
    let bad = format!(
        "{WEBHOOK}{}",
        entry(
            "a",
            "{ event = \"webhook.received\" }",
            "{ script = \"a.sh\", args = [\"{{webhook.commit}}\"] }"
        )
    );
    assert_eq!(fields(&bad), vec!["automations[0].run.args[0]"]);
}

#[test]
fn every_webhook_rule_names_its_field() {
    let cases = [
        (
            "id = \"not-a-uuid\"\ntitle = \"T\"\naction = \"event\"",
            "webhooks[0].id",
        ),
        (
            "id = \"7d3f2a4e-5b1c-4e8f-9a2d-6c0b1e3f4a5d\"\ntitle = \"\"\naction = \"event\"",
            "webhooks[0].title",
        ),
        (
            "id = \"7d3f2a4e-5b1c-4e8f-9a2d-6c0b1e3f4a5d\"\ntitle = \"T\"\naction = \"event\"\nvariables = [\"Bad-Name\"]",
            "webhooks[0].variables[0]",
        ),
        (
            "id = \"7d3f2a4e-5b1c-4e8f-9a2d-6c0b1e3f4a5d\"\ntitle = \"T\"\naction = \"shout\"",
            "webhooks[0].action",
        ),
        (
            "id = \"7d3f2a4e-5b1c-4e8f-9a2d-6c0b1e3f4a5d\"\ntitle = \"T\"\naction = \"script\"",
            "webhooks[0].run",
        ),
        (
            "id = \"7d3f2a4e-5b1c-4e8f-9a2d-6c0b1e3f4a5d\"\ntitle = \"T\"\naction = \"event\"\ntoken_sha256 = \"abc\"",
            "webhooks[0].token_sha256",
        ),
        (
            "id = \"7d3f2a4e-5b1c-4e8f-9a2d-6c0b1e3f4a5d\"\ntitle = \"T\"\naction = \"script\"\nrun = { script = \"a.sh\", args = [\"{{webhook.nope}}\"] }",
            "webhooks[0].run.args[0]",
        ),
    ];
    for (text, field) in cases {
        assert_eq!(
            fields(&format!("[[webhooks]]\n{text}\n")),
            vec![field],
            "{text}"
        );
    }
    let twice = format!("{WEBHOOK}{WEBHOOK}");
    assert_eq!(fields(&twice), vec!["webhooks[1].id"]);
}

#[test]
fn a_manual_automation_has_no_filters() {
    assert!(errors(&entry("a", "{ event = \"manual\" }", SCRIPT)).is_empty());
    assert_eq!(
        fields(&entry(
            "a",
            "{ event = \"manual\", services = [\"nas\"] }",
            SCRIPT
        )),
        vec!["automations[0].when.services"]
    );
}

#[test]
fn a_webhook_variable_may_not_take_the_name_of_a_fixed_field() {
    let text = "[[webhooks]]\nid = \"7d3f2a4e-5b1c-4e8f-9a2d-6c0b1e3f4a5d\"\ntitle = \"T\"\naction = \"event\"\nvariables = [\"id\", \"build2\"]\n";
    assert_eq!(fields(text), vec!["webhooks[0].variables[0]"]);
}

#[test]
fn a_variable_must_be_declared_by_every_chosen_webhook_and_a_missing_webhook_is_no_error() {
    let other = "[[webhooks]]\nid = \"0b9e8c2a-1d3f-4a5b-8c7d-9e0f1a2b3c4d\"\ntitle = \"Other\"\naction = \"event\"\n\n";
    let text = format!(
        "{WEBHOOK}{other}{}",
        entry(
            "a",
            "{ event = \"webhook.received\" }",
            "{ script = \"a.sh\", args = [\"{{webhook.branch}}\"] }"
        )
    );
    assert_eq!(fields(&text), vec!["automations[0].run.args[0]"]);
    let missing = format!(
        "{WEBHOOK}{}",
        entry(
            "a",
            "{ event = \"webhook.received\", webhooks = [\"00000000-0000-4000-8000-000000000000\"] }",
            SCRIPT
        )
    );
    assert!(fields(&missing).is_empty());
}

#[test]
fn tags_are_checked_one_by_one() {
    assert!(
        errors(&entry("a", STARTED, SCRIPT).replace(
            "title = \"T\"",
            "title = \"T\"\ntags = [\"media\", \"night\"]"
        ))
        .is_empty()
    );
    let text = entry("a", STARTED, SCRIPT).replace(
        "title = \"T\"",
        "title = \"T\"\ntags = [\"media\", \" \", \"Media\"]",
    );
    assert_eq!(
        fields(&text),
        vec!["automations[0].tags[1]", "automations[0].tags[2]"]
    );
    let hook = "[[webhooks]]\nid = \"7d3f2a4e-5b1c-4e8f-9a2d-6c0b1e3f4a5d\"\ntitle = \"T\"\naction = \"event\"\ntags = [\"\"]\n";
    assert_eq!(fields(hook), vec!["webhooks[0].tags[0]"]);
}

#[test]
fn a_webhook_reads_enabled_and_tags_through_its_marks() {
    let text = "[[webhooks]]\nid = \"7d3f2a4e-5b1c-4e8f-9a2d-6c0b1e3f4a5d\"\ntitle = \"T\"\nenabled = false\ntags = [\"ci\"]\naction = \"event\"\n";
    assert!(errors(text).is_empty());
    let webhooks = crate::services::decoded_webhooks(&text.parse().unwrap());
    assert!(!webhooks[0].enabled);
    assert_eq!(webhooks[0].tags, vec!["ci"]);
}
