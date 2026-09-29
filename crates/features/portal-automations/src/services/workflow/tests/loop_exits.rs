use serde_json::json;

use super::running::run;
use super::support::{fields, section};
use crate::services::workflow::workflow_errors;
use crate::types::{Ending, StepOutcome};

fn workflow(steps: &str) -> String {
    format!("[[workflows]]\nid = \"w\"\ntitle = \"W\"\ninputs = [\"service\"]\n{steps}")
}

#[tokio::test]
async fn a_break_leaves_its_loop_and_the_run_goes_on_after_it() {
    let outcome = run(&workflow("[[workflows.steps]]\nid = \"retry\"\nlabel = \"Retry\"\nkind = \"loop\"\nrepeat = 5\n[[workflows.steps.body]]\nid = \"ping\"\nkind = \"set\"\nvariable = \"pass\"\nvalue = \"{{loop.index}}\"\n[[workflows.steps.body]]\nid = \"worked\"\nkind = \"if\"\ncondition = { left = \"{{loop.index}}\", op = \"==\", right = \"1\" }\nthen = [{ id = \"done\", kind = \"break\" }]\n[[workflows.steps]]\nid = \"after\"\nkind = \"set\"\nvariable = \"after\"\nvalue = \"yes\"\n")).await;
    assert_eq!(outcome.ending, Ending::Succeeded(None));
    assert_eq!(
        outcome.frame.steps["retry"],
        json!({"iterations": 2, "left_early": true})
    );
    assert_eq!(outcome.frame.vars["after"], json!("yes"));
    let exit = outcome
        .trace
        .entries
        .iter()
        .find(|entry| entry.step == "done")
        .unwrap();
    assert_eq!(exit.outcome, StepOutcome::Succeeded);
    assert_eq!(exit.detail, "left the loop “Retry”");
}

#[tokio::test]
async fn a_continue_skips_the_rest_of_its_pass_only() {
    let outcome = run(&workflow("[[workflows.steps]]\nid = \"hosts\"\nkind = \"set\"\nvariable = \"hosts\"\nlist = [\"nas\", \"tv\", \"router\"]\n[[workflows.steps]]\nid = \"each\"\nlabel = \"Each\"\nkind = \"loop\"\nfor_each = \"{{vars.hosts}}\"\n[[workflows.steps.body]]\nid = \"skip\"\nkind = \"if\"\ncondition = { left = \"{{loop.item}}\", op = \"==\", right = \"tv\" }\nthen = [{ id = \"next\", kind = \"continue\" }]\n[[workflows.steps.body]]\nid = \"seen\"\nkind = \"log\"\nmessage = \"{{loop.item}}\"\n")).await;
    assert_eq!(outcome.ending, Ending::Succeeded(None));
    assert_eq!(
        outcome.frame.steps["each"],
        json!({"iterations": 3, "left_early": false})
    );
    let seen: Vec<&str> = outcome
        .trace
        .entries
        .iter()
        .filter(|entry| entry.step == "seen")
        .map(|entry| entry.detail.as_str())
        .collect();
    assert_eq!(seen, vec!["nas", "router"]);
    let next = outcome
        .trace
        .entries
        .iter()
        .find(|entry| entry.step == "next")
        .unwrap();
    assert_eq!(next.detail, "next pass of “Each”");
}

#[tokio::test]
async fn a_stop_inside_a_loop_still_ends_the_whole_run() {
    let outcome = run(&workflow("[[workflows.steps]]\nid = \"retry\"\nkind = \"loop\"\nrepeat = 3\n[[workflows.steps.body]]\nid = \"give_up\"\nkind = \"if\"\ncondition = { left = \"1\", op = \"==\", right = \"1\" }\nthen = [{ id = \"end\", kind = \"stop\", outcome = \"failed\", reason = \"gave up\" }]\n[[workflows.steps]]\nid = \"after\"\nkind = \"set\"\nvariable = \"after\"\nvalue = \"yes\"\n")).await;
    assert_eq!(outcome.ending, Ending::Failed("gave up".to_string()));
    assert!(!outcome.frame.vars.contains_key("after"));
}

#[tokio::test]
async fn steps_after_a_break_in_the_same_list_never_run() {
    let outcome = run(&workflow("[[workflows.steps]]\nid = \"once\"\nkind = \"loop\"\nrepeat = 3\n[[workflows.steps.body]]\nid = \"out\"\nkind = \"break\"\n[[workflows.steps.body]]\nid = \"never\"\nkind = \"set\"\nvariable = \"never\"\nvalue = \"ran\"\n")).await;
    assert_eq!(outcome.ending, Ending::Succeeded(None));
    assert!(!outcome.frame.vars.contains_key("never"));
    assert!(
        outcome
            .trace
            .entries
            .iter()
            .all(|entry| entry.step != "never")
    );
    assert_eq!(
        outcome.frame.steps["once"],
        json!({"iterations": 1, "left_early": true})
    );
}

#[test]
fn a_break_at_the_root_needs_an_enclosing_loop() {
    let text = workflow("[[workflows.steps]]\nid = \"out\"\nkind = \"break\"\n");
    assert_eq!(fields(&text), vec!["workflows[0].steps[0]"]);
    let message = workflow_errors(&section(&text))[0].message.clone();
    assert!(message.contains("needs an enclosing loop"), "{message}");
}

#[test]
fn a_continue_in_a_parallel_branch_inside_a_loop_is_refused() {
    let text = workflow(
        "[[workflows.steps]]\nid = \"each\"\nkind = \"loop\"\nrepeat = 2\n[[workflows.steps.body]]\nid = \"fan\"\nkind = \"parallel\"\nbranches = [[{ id = \"next\", kind = \"continue\" }], [{ id = \"idle\", kind = \"nothing\" }]]\n",
    );
    assert_eq!(
        fields(&text),
        vec!["workflows[0].steps[0].body[0].branches[0][0]"]
    );
}

#[test]
fn a_break_inside_an_if_inside_a_loop_is_allowed() {
    let text = workflow(
        "[[workflows.steps]]\nid = \"each\"\nkind = \"loop\"\nrepeat = 2\n[[workflows.steps.body]]\nid = \"check\"\nkind = \"if\"\ncondition = { left = \"1\", op = \"==\", right = \"1\" }\nthen = [{ id = \"out\", kind = \"break\" }]\nelse = [{ id = \"next\", kind = \"continue\" }]\n",
    );
    assert!(fields(&text).is_empty(), "{:?}", fields(&text));
}
