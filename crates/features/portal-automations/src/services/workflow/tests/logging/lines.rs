use super::super::running::run;
use crate::types::TraceEntry;

fn workflow(steps: &str) -> String {
    format!("[[workflows]]\nid = \"w\"\ntitle = \"W\"\ninputs = [\"service\"]\n{steps}")
}

fn lines<'a>(entries: &'a [TraceEntry], step: &str) -> Vec<&'a str> {
    entries
        .iter()
        .find(|entry| entry.step == step)
        .unwrap()
        .log
        .lines
        .iter()
        .map(String::as_str)
        .collect()
}

#[tokio::test]
async fn the_branch_an_if_took_is_logged_with_both_sides() {
    let outcome = run(&workflow("[[workflows.steps]]\nid = \"status\"\nkind = \"status\"\nservice = \"{{inputs.service}}\"\n[[workflows.steps]]\nid = \"check\"\nkind = \"if\"\ncondition = { left = \"{{steps.status.state}}\", op = \"==\", right = \"up\" }\nthen = [{ id = \"yes\", kind = \"nothing\" }]\nelse = [{ id = \"no\", kind = \"nothing\" }]\n")).await;
    let entries = &outcome.trace.entries;
    assert_eq!(
        lines(entries, "check"),
        vec![r#""down" equals "up": no"#, "took else"]
    );
    assert_eq!(lines(entries, "status"), vec!["nas → down"]);
}

#[tokio::test]
async fn a_loop_logs_its_mode_every_pass_and_its_end() {
    let outcome = run(&workflow("[[workflows.steps]]\nid = \"each\"\nkind = \"loop\"\nfor_each = \"{{inputs.service | split(',')}}\"\nbody = [{ id = \"note\", kind = \"set\", variable = \"last\", value = \"{{loop.item}}\" }]\n")).await;
    let entries = &outcome.trace.entries;
    assert_eq!(
        lines(entries, "each"),
        vec![
            "for each: 1 items",
            r#"pass 1: "nas""#,
            "ended after 1 passes"
        ]
    );
    assert_eq!(lines(entries, "note"), vec![r#"last = "nas""#]);
}

#[tokio::test]
async fn a_transform_logs_the_value_after_each_operation() {
    let outcome = run(&workflow("[[workflows.steps]]\nid = \"shape\"\nkind = \"transform\"\ninput = \"a, b\"\noperations = [{ op = \"split\", args = [\",\"] }, { op = \"each\", operations = [{ op = \"trim\" }] }]\n")).await;
    assert_eq!(
        lines(&outcome.trace.entries, "shape"),
        vec![
            r#"after 1 (split): ["a"," b"]"#,
            r#"after 2 (each): ["a","b"]"#
        ]
    );
}

#[tokio::test]
async fn a_log_step_writes_its_message_with_its_level_and_never_ends_the_run() {
    let outcome = run(&workflow("[[workflows.steps]]\nid = \"each\"\nkind = \"loop\"\nfor_each = \"{{inputs.service | split(',')}}\"\n[[workflows.steps.body]]\nid = \"status\"\nkind = \"status\"\nservice = \"{{loop.item}}\"\n[[workflows.steps.body]]\nid = \"note\"\nkind = \"log\"\nmessage = \"checking {{loop.item}}: {{steps.status.state}}\"\nlevel = \"warning\"\n[[workflows.steps]]\nid = \"loud\"\nkind = \"log\"\nlevel = \"error\"\nmessage = \"still here\"\n[[workflows.steps]]\nid = \"after\"\nkind = \"nothing\"\n")).await;
    assert_eq!(outcome.ending, crate::types::Ending::Succeeded(None));
    let entries = &outcome.trace.entries;
    let note = entries.iter().find(|entry| entry.step == "note").unwrap();
    assert_eq!(note.detail, "checking nas: down");
    assert_eq!(note.level, Some(crate::types::LogLevel::Warning));
    assert_eq!(lines(entries, "note"), vec!["[warning] checking nas: down"]);
    assert_eq!(outcome.frame.steps["note"]["message"], "checking nas: down");
    assert!(entries.iter().any(|entry| entry.step == "after"));
}

#[test]
fn a_log_step_needs_a_message_and_a_known_level() {
    let fields = super::super::support::fields(&workflow(
        "[[workflows.steps]]\nid = \"note\"\nkind = \"log\"\nlevel = \"loud\"\n",
    ));
    assert_eq!(
        fields,
        vec![
            "workflows[0].steps[0].level",
            "workflows[0].steps[0].message"
        ]
    );
}
