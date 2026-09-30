use std::sync::Arc;

use tokio::sync::watch;

use super::super::running::{FakeActions, run, run_prepared};
use crate::services::workflow::{SecretLookup, Secrets};
use crate::types::TraceEntry;

fn workflow(steps: &str) -> String {
    format!("[[workflows]]\nid = \"w\"\ntitle = \"W\"\ninputs = [\"service\"]\n{steps}")
}

fn values(entry: &TraceEntry) -> Vec<(&str, &str)> {
    entry
        .log
        .values
        .iter()
        .map(|value| (value.template.as_str(), value.value.as_str()))
        .collect()
}

fn entry<'a>(entries: &'a [TraceEntry], step: &str, iteration: Option<usize>) -> &'a TraceEntry {
    entries
        .iter()
        .find(|entry| entry.step == step && entry.iteration == iteration)
        .unwrap()
}

#[tokio::test]
async fn a_loop_keeps_its_own_template_and_each_nested_step_keeps_its_own() {
    let outcome = run(&workflow("[[workflows.steps]]\nid = \"svc\"\nkind = \"set\"\nvariable = \"svc\"\nlist = [\"Media\", \"qTorrent\"]\n[[workflows.steps]]\nid = \"each\"\nkind = \"loop\"\nfor_each = \"{{vars.svc}}\"\n[[workflows.steps.body]]\nid = \"note\"\nkind = \"set\"\nvariable = \"last\"\nvalue = \"{{loop.item}} of {{inputs.service}}\"\n")).await;
    let entries = &outcome.trace.entries;
    assert_eq!(
        values(entry(entries, "each", None)),
        vec![("{{vars.svc}}", r#"["Media","qTorrent"]"#)]
    );
    assert_eq!(
        values(entry(entries, "note", Some(1))),
        vec![("{{loop.item}} of {{inputs.service}}", "\"qTorrent of nas\"")]
    );
    assert_eq!(
        entry(entries, "note", Some(0)).item.as_deref(),
        Some("\"Media\"")
    );
    assert!(values(entry(entries, "svc", None)).is_empty());
}

#[tokio::test]
async fn parallel_branches_keep_their_values_apart() {
    let outcome = run(&workflow("[[workflows.steps]]\nid = \"both\"\nkind = \"parallel\"\nbranches = [\n  [{ id = \"a\", kind = \"set\", variable = \"a\", value = \"{{inputs.service}}-a\" }, { id = \"nap\", kind = \"wait\", seconds = 1 }],\n  [{ id = \"b\", kind = \"set\", variable = \"b\", value = \"{{inputs.service}}-b\" }],\n]\n")).await;
    let entries = &outcome.trace.entries;
    assert_eq!(
        values(entry(entries, "a", None)),
        vec![("{{inputs.service}}-a", "\"nas-a\"")]
    );
    assert_eq!(
        values(entry(entries, "b", None)),
        vec![("{{inputs.service}}-b", "\"nas-b\"")]
    );
    assert!(values(entry(entries, "both", None)).is_empty());
}

#[tokio::test]
async fn secrets_stay_masked_in_the_log() {
    let lookup: SecretLookup = Arc::new(|key: &str| (key == "token").then(|| "s3cr3t".to_string()));
    let (_sender, stop) = watch::channel(false);
    let outcome = run_prepared(
        &workflow("[[workflows.steps]]\nid = \"auth\"\nkind = \"set\"\nvariable = \"header\"\nvalue = \"Bearer {{secrets.token}}\"\n"),
        (Arc::new(FakeActions::default()), stop),
        &[],
        Secrets::new(lookup),
    )
    .await;
    let entry = entry(&outcome.trace.entries, "auth", None);
    assert_eq!(
        values(entry),
        vec![("Bearer {{secrets.token}}", "\"Bearer ***\"")]
    );
    assert!(
        entry.log.lines.iter().all(|line| !line.contains("s3cr3t")),
        "{:?}",
        entry.log.lines
    );
}

async fn masked_value(secret: &'static str, template: &str) -> String {
    let lookup: SecretLookup =
        Arc::new(move |key: &str| (key == "token").then(|| secret.to_string()));
    let (_sender, stop) = watch::channel(false);
    let outcome = run_prepared(
        &workflow(&format!(
            "[[workflows.steps]]\nid = \"auth\"\nkind = \"set\"\nvariable = \"header\"\nvalue = \"{template}\"\n"
        )),
        (Arc::new(FakeActions::default()), stop),
        &[],
        Secrets::new(lookup),
    )
    .await;
    let entry = entry(&outcome.trace.entries, "auth", None);
    entry.log.values[0].value.clone()
}

#[tokio::test]
async fn a_secret_across_the_cut_is_masked_before_the_value_is_cut() {
    let padding = "x".repeat(285);
    let value = masked_value(
        "abcdefghijklmnopqrst",
        &format!("{padding}{{{{secrets.token}}}}"),
    )
    .await;
    assert!(!value.contains("abcdefgh"), "{value}");
    assert!(value.contains("***"), "{value}");
}

#[tokio::test]
async fn a_secret_with_a_quote_is_masked_in_its_escaped_form() {
    let value = masked_value("pa\"ss\\word", "Bearer {{secrets.token}}").await;
    assert_eq!(value, "\"Bearer ***\"");
}

#[test]
fn the_edge_of_a_cut_output_hides_the_part_of_a_secret_it_holds() {
    let secrets = Secrets::new(Arc::new(|_: &str| Some("0123456789abcdef".to_string())));
    secrets.reveal("token").unwrap();
    assert_eq!(secrets.mask("9abcdef and more"), "*** and more");
    assert_eq!(secrets.mask("text then 012345"), "text then ***");
    assert_eq!(secrets.mask("tail 0123456789abcdef"), "tail ***");
}
