use super::support::{fields, section};
use crate::services::workflow::{decode_workflow, workflow_errors};
use crate::types::{Condition, LoopMode, Operator, StepKind};

pub const REVIVE: &str = r#"
[[workflows]]
id = "revive"
title = "Revive a service"
inputs = ["service"]

[[workflows.steps]]
id = "probe"
kind = "probe"
service = "{{inputs.service}}"

[[workflows.steps]]
id = "check"
kind = "if"
condition = { left = "{{steps.probe.state}}", op = "!=", right = "up" }

[[workflows.steps.then]]
id = "retry"
kind = "loop"
repeat = 3

[[workflows.steps.then.body]]
id = "ping"
kind = "http"
url = "http://nas.lan/api/ping"
fail_on_error = false

[[workflows.steps.then.body]]
id = "pause"
kind = "wait"
seconds = 5

[[workflows.steps]]
id = "notify"
kind = "telegram"
text = "{{inputs.service}} is {{steps.probe.state}}"
"#;

#[test]
fn a_valid_workflow_loads_with_its_nested_steps() {
    assert!(fields(REVIVE).is_empty(), "{:?}", fields(REVIVE));
    let workflow = decode_workflow(&section(REVIVE).workflows[0]).unwrap();
    assert_eq!(workflow.steps.len(), 3);
    assert_eq!(workflow.timeout_seconds, 300);
    let StepKind::If {
        condition,
        then,
        otherwise,
    } = &workflow.steps[1].kind
    else {
        panic!("not an if");
    };
    assert!(otherwise.is_empty());
    assert!(matches!(
        condition,
        Condition::Compare {
            operator: Operator::NotEqual,
            ..
        }
    ));
    let StepKind::Loop {
        mode,
        max_iterations,
        body,
    } = &then[0].kind
    else {
        panic!("not a loop");
    };
    assert_eq!(*mode, LoopMode::Repeat(3));
    assert_eq!(*max_iterations, 100);
    assert_eq!(body.len(), 2);
}

#[test]
fn a_nested_error_is_named_by_its_path() {
    let text = format!(
        "{REVIVE}\n[[workflows]]\nid = \"second\"\ntitle = \"S\"\n[[workflows.steps]]\nid = \"a\"\nkind = \"wait\"\nseconds = 1\n[[workflows.steps]]\nid = \"b\"\nkind = \"wait\"\nseconds = 1\n[[workflows.steps]]\nid = \"c\"\nkind = \"if\"\ncondition = {{ left = \"1\", op = \"is-empty\" }}\n[[workflows.steps.then]]\nid = \"d\"\nkind = \"http\"\nurl = \"ftp://x\"\n"
    );
    assert_eq!(fields(&text), vec!["workflows[1].steps[2].then[0].url"]);
}

#[test]
fn nesting_nine_containers_is_refused_at_the_ninth() {
    let mut text = String::from("[[workflows]]\nid = \"deep\"\ntitle = \"D\"\n");
    let mut header = String::from("workflows.steps");
    for level in 0..9 {
        text.push_str(&format!(
            "[[{header}]]\nid = \"if{level}\"\nkind = \"if\"\ncondition = {{ left = \"a\", op = \"==\", right = \"a\" }}\n"
        ));
        header.push_str(".then");
    }
    text.push_str(&format!(
        "[[{header}]]\nid = \"leaf\"\nkind = \"wait\"\nseconds = 1\n"
    ));
    let found = fields(&text);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(
        found[0],
        format!("workflows[0].steps[0]{}", ".then[0]".repeat(8))
    );
}

#[test]
fn calling_in_a_cycle_is_refused_at_the_closing_step() {
    let text = "[[workflows]]\nid = \"a\"\ntitle = \"A\"\n[[workflows.steps]]\nid = \"call_b\"\nkind = \"workflow\"\nworkflow = \"b\"\n\n[[workflows]]\nid = \"b\"\ntitle = \"B\"\n[[workflows.steps]]\nid = \"call_a\"\nkind = \"workflow\"\nworkflow = \"a\"\n";
    let found = workflow_errors(&section(text));
    assert!(!found.is_empty());
    assert!(
        found.iter().all(|error| error.message.contains("cycle")),
        "{found:?}"
    );
    assert!(
        found
            .iter()
            .any(|error| error.field == "workflows[1].steps[0].workflow")
    );
}

#[test]
fn a_call_chain_deeper_than_four_is_refused() {
    let mut text = String::new();
    for index in 0..6 {
        text.push_str(&format!(
            "[[workflows]]\nid = \"w{index}\"\ntitle = \"W\"\n"
        ));
        if index < 5 {
            text.push_str(&format!(
                "[[workflows.steps]]\nid = \"call\"\nkind = \"workflow\"\nworkflow = \"w{}\"\n",
                index + 1
            ));
        } else {
            text.push_str("[[workflows.steps]]\nid = \"rest\"\nkind = \"wait\"\nseconds = 1\n");
        }
    }
    assert_eq!(fields(&text), vec!["workflows[0].steps[0].workflow"]);
}

#[test]
fn every_broken_rule_names_its_field() {
    let cases = [
        ("id = \"x\"\nkind = \"teleport\"", "steps[0].kind"),
        ("id = \"X\"\nkind = \"wait\"\nseconds = 1", "steps[0].id"),
        (
            "id = \"x\"\nkind = \"wait\"\nseconds = 0",
            "steps[0].seconds",
        ),
        (
            "id = \"x\"\nkind = \"loop\"\nrepeat = 2\nfor_each = \"[]\"\nbody = [{ id = \"y\", kind = \"wait\", seconds = 1 }]",
            "steps[0]",
        ),
        (
            "id = \"x\"\nkind = \"loop\"\nrepeat = 101\nbody = [{ id = \"y\", kind = \"wait\", seconds = 1 }]",
            "steps[0].repeat",
        ),
        (
            "id = \"x\"\nkind = \"parallel\"\nbranches = [[{ id = \"y\", kind = \"wait\", seconds = 1 }]]",
            "steps[0].branches",
        ),
        (
            "id = \"x\"\nkind = \"http\"\nurl = \"http://a\"\nheaders = { Host = \"b\" }",
            "steps[0].headers.Host",
        ),
        (
            "id = \"x\"\nkind = \"http\"\nurl = \"http://a\"\ntimeout_seconds = 61",
            "steps[0].timeout_seconds",
        ),
        (
            "id = \"x\"\nkind = \"http\"\nurl = \"http://a\"\nmethod = \"BREW\"",
            "steps[0].method",
        ),
        (
            "id = \"x\"\nkind = \"script\"\nscript = \"/bin/sh\"",
            "steps[0].script",
        ),
        (
            "id = \"x\"\nkind = \"stop\"\noutcome = \"maybe\"",
            "steps[0].outcome",
        ),
        ("id = \"x\"\nkind = \"set\"\nvariable = \"v\"", "steps[0]"),
        ("id = \"x\"\nkind = \"telegram\"", "steps[0].text"),
        ("id = \"x\"\nkind = \"probe\"", "steps[0].service"),
        (
            "id = \"x\"\nkind = \"if\"\ncondition = { left = \"a\", op = \"~\" , right = \"b\" }\nthen = [{ id = \"y\", kind = \"wait\", seconds = 1 }]",
            "steps[0].condition.op",
        ),
        (
            "id = \"x\"\nkind = \"workflow\"\nworkflow = \"ghost\"",
            "steps[0].workflow",
        ),
    ];
    for (step, field) in cases {
        let text =
            format!("[[workflows]]\nid = \"w\"\ntitle = \"W\"\n[[workflows.steps]]\n{step}\n");
        assert_eq!(
            fields(&text),
            vec![format!("workflows[0].{field}")],
            "{step}"
        );
    }
}

#[test]
fn entry_rules_and_duplicates_are_named() {
    let text = "[[workflows]]\nid = \"runs\"\ntitle = \"\"\ninputs = [\"a\", \"a\"]\ntimeout_seconds = 0\nsteps = []\n\n[[workflows]]\nid = \"twice\"\ntitle = \"T\"\n[[workflows.steps]]\nid = \"s\"\nkind = \"wait\"\nseconds = 1\n[[workflows.steps]]\nid = \"s\"\nkind = \"wait\"\nseconds = 1\n";
    let mut found = fields(text);
    found.sort();
    assert_eq!(
        found,
        vec![
            "workflows[0].id",
            "workflows[0].inputs[1]",
            "workflows[0].steps",
            "workflows[0].timeout_seconds",
            "workflows[0].title",
            "workflows[1].steps[1].id",
        ]
    );
}

#[test]
fn a_call_with_an_undeclared_input_is_refused() {
    let text = format!(
        "{REVIVE}\n[[workflows]]\nid = \"caller\"\ntitle = \"C\"\n[[workflows.steps]]\nid = \"go\"\nkind = \"workflow\"\nworkflow = \"revive\"\ninputs = {{ service = \"nas\", colour = \"red\" }}\n"
    );
    assert_eq!(fields(&text), vec!["workflows[1].steps[0].inputs.colour"]);
}

#[test]
fn a_response_sample_must_be_json_of_at_most_sixteen_kibibytes_and_nothing_takes_no_fields() {
    let step = |sample: &str| {
        format!(
            "[[workflows]]\nid = \"w\"\ntitle = \"W\"\n[[workflows.steps]]\nid = \"ping\"\nkind = \"http\"\nurl = \"http://nas.lan\"\nresponse_sample = '''{sample}'''\n[[workflows.steps]]\nid = \"idle\"\nkind = \"nothing\"\n"
        )
    };
    assert_eq!(
        fields(&step(r#"{"state":"up","items":[1,2]}"#)),
        Vec::<String>::new()
    );
    assert_eq!(
        fields(&step("<html>")),
        vec!["workflows[0].steps[0].response_sample"]
    );
    let large = format!(r#"{{"text":"{}"}}"#, "x".repeat(17 * 1024));
    assert_eq!(
        fields(&step(&large)),
        vec!["workflows[0].steps[0].response_sample"]
    );
}
