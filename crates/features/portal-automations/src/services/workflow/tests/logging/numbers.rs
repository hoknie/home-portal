use std::time::Instant;

use super::super::running::run;
use super::super::support::fields;
use crate::types::{Ending, KINDS, kind_named};

fn workflow(steps: &str) -> String {
    format!("[[workflows]]\nid = \"w\"\ntitle = \"W\"\ninputs = [\"service\"]\n{steps}")
}

#[tokio::test]
async fn a_wait_from_a_variable_waits_that_long() {
    let began = Instant::now();
    let outcome = run(&workflow("[[workflows.steps]]\nid = \"pause\"\nkind = \"set\"\nvariable = \"pause\"\njson = \"1\"\n[[workflows.steps]]\nid = \"nap\"\nkind = \"wait\"\nseconds = \"{{vars.pause}}\"\n")).await;
    assert_eq!(outcome.ending, Ending::Succeeded(None));
    assert!(began.elapsed().as_millis() >= 900);
    let nap = outcome
        .trace
        .entries
        .iter()
        .find(|entry| entry.step == "nap")
        .unwrap();
    assert_eq!(nap.log.lines, vec!["waited 1 s"]);
}

#[tokio::test]
async fn a_number_out_of_bounds_from_a_template_fails_naming_the_field_and_the_bounds() {
    let outcome = run(&workflow("[[workflows.steps]]\nid = \"tries\"\nkind = \"set\"\nvariable = \"tries\"\nvalue = \"500\"\n[[workflows.steps]]\nid = \"again\"\nkind = \"loop\"\nrepeat = \"{{vars.tries}}\"\nbody = [{ id = \"n\", kind = \"nothing\" }]\n")).await;
    assert_eq!(
        outcome.ending,
        Ending::Failed(
            "repeat is 500 ({{vars.tries}}), it must be a whole number from 1 to 100".into()
        )
    );
}

#[test]
fn literals_keep_their_load_checks_and_pickers_stay_literal() {
    assert_eq!(
        fields(&workflow(
            "[[workflows.steps]]\nid = \"nap\"\nkind = \"wait\"\nseconds = 5000\n"
        )),
        vec!["workflows[0].steps[0].seconds"]
    );
    assert_eq!(
        fields(&workflow(
            "[[workflows.steps]]\nid = \"nap\"\nkind = \"wait\"\nseconds = \"soon\"\n"
        )),
        vec!["workflows[0].steps[0].seconds"]
    );
    assert_eq!(
        fields(&workflow(
            "[[workflows.steps]]\nid = \"nap\"\nkind = \"wait\"\nseconds = \"{{vars.missing}}\"\n"
        )),
        vec!["workflows[0].steps[0].seconds"]
    );
    assert_eq!(
        fields(&workflow(
            "[[workflows.steps]]\nid = \"call\"\nkind = \"workflow\"\nworkflow = \"{{inputs.service}}\"\n"
        )),
        vec!["workflows[0].steps[0].workflow"]
    );
}

#[test]
fn the_catalogue_marks_number_fields_and_table_keys_that_take_templates() {
    let field = |kind: &str, name: &str| {
        *kind_named(kind)
            .unwrap()
            .fields
            .iter()
            .find(|field| field.name == name)
            .unwrap()
    };
    assert!(field("loop", "repeat").templated && field("loop", "max_iterations").templated);
    assert!(
        field("wait", "seconds").templated
            && field("http", "timeout_seconds").templated
            && field("script", "timeout_seconds").templated
    );
    assert!(field("set", "object").template_keys && field("http", "headers").template_keys);
    assert!(!field("script", "env").template_keys && !field("workflow", "inputs").template_keys);
    assert_eq!(
        KINDS
            .iter()
            .flat_map(|kind| kind.fields)
            .filter(|field| field.templated)
            .count(),
        5
    );
}
