use std::sync::Arc;

use serde_json::json;

use super::running::run;
use super::support::section;
use crate::services::workflow::{Frame, Secrets, render_text, render_value, workflow_errors};
use crate::types::{Ending, FILTERS, filter_named};

fn frame() -> Frame {
    let mut frame = Frame::new(
        Arc::new(Vec::new().into()),
        [("service".to_string(), serde_json::json!("nas"))].into(),
        Arc::new(Secrets::none()),
    );
    frame.steps.insert(
        "list".to_string(),
        json!({"status": 200, "json": {"disks": [{"name": "sda", "size": 2}, {"name": "sdb", "size": 3.5}]}}),
    );
    frame
}

fn errors(steps: &str) -> Vec<(String, String)> {
    workflow_errors(&section(&format!(
        "[[workflows]]\nid = \"w\"\ntitle = \"W\"\ninputs = [\"service\"]\n{steps}"
    )))
    .into_iter()
    .map(|error| (error.field, error.message))
    .collect()
}

#[test]
fn a_chain_of_filters_runs_left_to_right() {
    let frame = frame();
    assert_eq!(
        render_text(
            "{{steps.list.json.disks | pluck(\"name\") | join(\", \") | upper}}",
            &frame
        )
        .unwrap(),
        "SDA, SDB"
    );
    assert_eq!(
        render_value("{{ steps.list.json.disks | pluck('size') | sum }}", &frame).unwrap(),
        json!(5.5)
    );
    assert_eq!(
        render_text(
            "{{inputs.service|upper}} has {{steps.list.json.disks|length}}",
            &frame
        )
        .unwrap(),
        "NAS has 2"
    );
}

#[test]
fn a_default_fills_a_missing_value_and_other_filters_pass_it_through() {
    let frame = frame();
    assert_eq!(
        render_text("{{vars.note | default(\"none\")}}", &frame).unwrap(),
        "none"
    );
    assert_eq!(
        render_text("[{{vars.note | upper}}]", &frame).unwrap(),
        "[]"
    );
}

#[test]
fn a_filter_on_the_wrong_type_fails_naming_what_it_takes_and_got() {
    let frame = frame();
    let error = render_text("{{steps.list.status | join(\", \")}}", &frame).unwrap_err();
    assert_eq!(error, "join takes a list and got a number");
}

#[test]
fn every_filter_of_the_catalogue_has_an_evaluator() {
    for filter in FILTERS {
        let call = crate::types::FilterCall::literal(filter.name, Vec::new());
        let error =
            crate::services::workflow::apply_chain(json!(12345), &[call], &|_| Ok(json!(null)))
                .err()
                .unwrap_or_default();
        assert!(
            !error.contains("is not a filter"),
            "{}: {error}",
            filter.name
        );
    }
    assert!(filter_named("shout").is_none());
}

#[test]
fn an_unknown_filter_is_refused_at_load_naming_the_field_and_the_filter() {
    let found = errors(
        "[[workflows.steps]]\nid = \"say\"\nkind = \"notify\"\ntext = \"{{inputs.service | shout}}\"\n",
    );
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].0, "workflows[0].steps[0].text");
    assert!(found[0].1.contains("shout"), "{}", found[0].1);
}

#[test]
fn argument_counts_literal_types_and_certain_mismatches_are_refused_at_load() {
    let found = errors(
        "[[workflows.steps]]\nid = \"ping\"\nkind = \"http\"\nurl = \"http://nas\"\n[[workflows.steps]]\nid = \"a\"\nkind = \"notify\"\ntext = \"{{inputs.service | join(',')}}\"\n[[workflows.steps]]\nid = \"b\"\nkind = \"notify\"\ntext = \"{{inputs.service | split(3)}}\"\n[[workflows.steps]]\nid = \"c\"\nkind = \"notify\"\ntext = \"{{steps.ping.status | upper}}\"\n[[workflows.steps]]\nid = \"d\"\nkind = \"notify\"\ntext = \"{{steps.ping.json | pluck(\\\"name\\\") | join(\\\", \\\")}} {{inputs.service | replace(\\\"a\\\", \\\"b\\\") | split(\\\",\\\") | length}}\"\n[[workflows.steps]]\nid = \"e\"\nkind = \"notify\"\ntext = \"{{inputs.service | join(}}\"\n",
    );
    let messages: Vec<&str> = found.iter().map(|(_, message)| message.as_str()).collect();
    assert_eq!(found.len(), 4, "{messages:?}");
    assert!(
        messages[0].contains("join takes a list and got text"),
        "{}",
        messages[0]
    );
    assert!(
        messages[1].contains("separator of split must be quoted text"),
        "{}",
        messages[1]
    );
    assert!(
        messages[2].contains("upper takes text and got a number"),
        "{}",
        messages[2]
    );
    assert!(messages[3].contains("cannot be read"), "{}", messages[3]);
}

#[test]
fn item_and_index_exist_only_inside_a_transform() {
    let found =
        errors("[[workflows.steps]]\nid = \"say\"\nkind = \"notify\"\ntext = \"{{item.name}}\"\n");
    assert_eq!(found.len(), 1);
    assert!(found[0].1.contains("transform"), "{}", found[0].1);
}

#[tokio::test]
async fn a_filter_failing_at_run_time_fails_its_step() {
    let outcome = run("[[workflows]]\nid = \"w\"\ntitle = \"W\"\n[[workflows.steps]]\nid = \"v\"\nkind = \"set\"\nvariable = \"n\"\nvalue = \"abc\"\n[[workflows.steps]]\nid = \"say\"\nkind = \"notify\"\ntext = \"{{vars.n | number}}\"\n").await;
    assert_eq!(
        outcome.ending,
        Ending::Failed("number cannot read \"abc\" as a number".to_string())
    );
}

#[test]
fn an_element_filter_after_split_is_accepted_at_load_and_maps_the_list() {
    assert!(
        errors("[[workflows.steps]]\nid = \"say\"\nkind = \"notify\"\ntext = \"{{inputs.service | split(',') | upper | join(' ')}}\"\n").is_empty()
    );
    assert_eq!(
        render_value("{{inputs.service | split('a') | upper}}", &frame()).unwrap(),
        json!(["N", "S"])
    );
}
