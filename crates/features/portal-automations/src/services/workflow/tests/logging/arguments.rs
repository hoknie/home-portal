use std::sync::Arc;

use serde_json::json;

use super::super::running::run;
use super::super::support::fields;
use crate::services::workflow::{Frame, Secrets, render_text, render_value, workflow_errors};

fn frame() -> Frame {
    let mut frame = Frame::new(
        Arc::new(Vec::new()),
        [("which".to_string(), json!("nas"))].into(),
        Arc::new(Secrets::none()),
    );
    frame.steps.insert(
        "states".to_string(),
        json!({"json": {"media": "up", "nas": "down", "list": [1, 2]}}),
    );
    frame
}

#[test]
fn a_filter_argument_may_be_a_name_read_when_the_template_renders() {
    let frame = frame();
    assert_eq!(
        render_text("{{steps.states.json | get(inputs.which)}}", &frame).unwrap(),
        "down"
    );
    assert_eq!(
        render_value("{{steps.states.json | get(\"media\")}}", &frame).unwrap(),
        json!("up")
    );
    let wrong = render_text(
        "{{steps.states.json | get(steps.states.json.list)}}",
        &frame,
    )
    .unwrap_err();
    assert!(
        wrong.contains("the argument key of get is a list"),
        "{wrong}"
    );
}

#[tokio::test]
async fn a_key_from_the_loop_reads_a_different_value_in_every_pass() {
    let outcome = run("[[workflows]]\nid = \"w\"\ntitle = \"W\"\n[[workflows.steps]]\nid = \"states\"\nkind = \"set\"\nvariable = \"states\"\njson = '{\"media\": \"up\", \"nas\": \"down\"}'\n[[workflows.steps]]\nid = \"start\"\nkind = \"set\"\nvariable = \"seen\"\nvalue = \"\"\n[[workflows.steps]]\nid = \"each\"\nkind = \"loop\"\nfor_each = '{{vars.states | keys}}'\nbody = [{ id = \"note\", kind = \"set\", variable = \"seen\", value = \"{{vars.seen}} {{loop.item}}={{vars.states | get(loop.item)}}\" }]\n").await;
    assert_eq!(outcome.frame.vars["seen"], json!(" media=up nas=down"));
}

#[test]
fn an_argument_name_out_of_scope_is_refused_and_a_word_that_is_no_name_stays_unreadable() {
    let text = "[[workflows]]\nid = \"w\"\ntitle = \"W\"\n[[workflows.steps]]\nid = \"note\"\nkind = \"log\"\nmessage = \"{{steps.missing.json | get(loop.item)}}\"\n";
    let errors = workflow_errors(&super::super::support::section(text));
    assert_eq!(errors.len(), 1);
    assert!(
        errors[0].message.contains("argument of get"),
        "{}",
        errors[0].message
    );
    assert!(
        errors[0].message.contains("loop.item"),
        "{}",
        errors[0].message
    );
    assert_eq!(
        fields(
            "[[workflows]]\nid = \"w\"\ntitle = \"W\"\n[[workflows.steps]]\nid = \"note\"\nkind = \"log\"\nmessage = \"{{portal.services | get(name)}}\"\n"
        ),
        vec!["workflows[0].steps[0].message"]
    );
}

#[tokio::test]
async fn get_by_key_from_a_variable_in_a_transform_renders_its_argument_and_logs_it() {
    let outcome = run("[[workflows]]\nid = \"w\"\ntitle = \"W\"\n[[workflows.steps]]\nid = \"which\"\nkind = \"set\"\nvariable = \"which\"\nvalue = \"nas\"\n[[workflows.steps]]\nid = \"pick\"\nkind = \"transform\"\ninput = '{{vars.which | default(\"\")}}'\noperations = [{ op = \"default\", args = [\"x\"] }]\n[[workflows.steps]]\nid = \"shape\"\nkind = \"set\"\nvariable = \"counts\"\njson = '{\"media\": 1, \"nas\": 2}'\n[[workflows.steps]]\nid = \"one\"\nkind = \"transform\"\ninput = \"{{vars.counts}}\"\noperations = [{ op = \"get\", args = [\"{{vars.which}}\"] }]\n").await;
    assert_eq!(outcome.frame.steps["one"]["value"], json!(2));
    let entry = outcome
        .trace
        .entries
        .iter()
        .find(|entry| entry.step == "one")
        .unwrap();
    assert!(
        entry
            .log
            .values
            .iter()
            .any(|value| value.template == "{{vars.which}}" && value.value == "\"nas\""),
        "{:?}",
        entry.log.values
    );
    let refused = workflow_errors(&super::super::support::section(
        "[[workflows]]\nid = \"w\"\ntitle = \"W\"\n[[workflows.steps]]\nid = \"one\"\nkind = \"transform\"\ninput = \"{}\"\noperations = [{ op = \"get\", args = [\"{{vars.nowhere}}\"] }]\n",
    ));
    assert_eq!(
        refused[0].field,
        "workflows[0].steps[0].operations[0].args[0]"
    );
}
