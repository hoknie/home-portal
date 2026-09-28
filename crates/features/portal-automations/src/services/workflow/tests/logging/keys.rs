use serde_json::json;

use super::super::running::run;
use super::super::support::fields;
use crate::types::Ending;

fn workflow(steps: &str) -> String {
    format!("[[workflows]]\nid = \"w\"\ntitle = \"W\"\ninputs = [\"service\"]\n{steps}")
}

#[tokio::test]
async fn a_dictionary_keyed_by_service_ids_gets_one_key_per_pass() {
    let outcome = run(&workflow("[[workflows.steps]]\nid = \"each\"\nkind = \"loop\"\nfor_each = \"{{portal.services}}\"\n[[workflows.steps.body]]\nid = \"states\"\nkind = \"set\"\nvariable = \"states\"\nobject = { \"{{loop.item.id}}\" = \"{{loop.item.state}}\", fixed = \"x\" }\n")).await;
    assert_eq!(outcome.ending, Ending::Succeeded(None));
    assert_eq!(
        outcome.frame.vars["states"],
        json!({"nas": "down", "fixed": "x"})
    );
    let first = outcome
        .trace
        .entries
        .iter()
        .find(|entry| entry.step == "states")
        .unwrap();
    assert!(
        first
            .log
            .values
            .iter()
            .any(|value| value.template == "{{loop.item.id}}" && value.value == "\"media\""),
        "{:?}",
        first.log.values
    );
}

#[tokio::test]
async fn two_keys_that_collide_fail_the_step_naming_both() {
    let outcome = run(&workflow("[[workflows.steps]]\nid = \"both\"\nkind = \"set\"\nvariable = \"both\"\nobject = { \"{{inputs.service}}\" = \"1\", nas = \"2\" }\n")).await;
    let Ending::Failed(reason) = outcome.ending else {
        panic!("{:?}", outcome.ending)
    };
    assert!(
        reason.contains("{{inputs.service}}")
            && reason.contains("nas")
            && reason.contains("both render"),
        "{reason}"
    );
}

#[tokio::test]
async fn a_header_name_that_is_not_valid_after_rendering_fails_the_step() {
    let outcome = run(&workflow("[[workflows.steps]]\nid = \"ask\"\nkind = \"http\"\nurl = \"http://127.0.0.1:9\"\nheaders = { \"{{inputs.service}} name\" = \"x\" }\n")).await;
    let Ending::Failed(reason) = outcome.ending else {
        panic!("{:?}", outcome.ending)
    };
    assert!(
        reason.contains("the header name \"nas name\" is not valid"),
        "{reason}"
    );
}

#[test]
fn an_automation_field_name_stays_literal() {
    assert_eq!(
        fields(&workflow(
            "[[workflows.steps]]\nid = \"start\"\nkind = \"automation\"\nautomation = \"x\"\nfields = { \"{{inputs.service}}\" = \"1\" }\n"
        )),
        vec!["workflows[0].steps[0].fields.{{inputs.service}}"]
    );
}
