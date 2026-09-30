use super::support::fields;

fn one(steps: &str) -> Vec<String> {
    fields(&format!(
        "[[workflows]]\nid = \"w\"\ntitle = \"W\"\ninputs = [\"service\"]\n{steps}"
    ))
}

#[test]
fn a_later_step_is_not_in_scope() {
    let found = one(
        "[[workflows.steps]]\nid = \"first\"\nkind = \"telegram\"\ntext = \"{{steps.second.status}}\"\n[[workflows.steps]]\nid = \"second\"\nkind = \"http\"\nurl = \"http://a\"\n",
    );
    assert_eq!(found, vec!["workflows[0].steps[0].text"]);
}

#[test]
fn steps_inside_earlier_branches_and_enclosing_blocks_are_in_scope() {
    let found = one(
        "[[workflows.steps]]\nid = \"both\"\nkind = \"parallel\"\nbranches = [[{ id = \"a\", kind = \"status\", service = \"nas\" }], [{ id = \"b\", kind = \"status\", service = \"{{inputs.service}}\" }]]\n[[workflows.steps]]\nid = \"again\"\nkind = \"loop\"\nrepeat = 2\nbody = [{ id = \"say\", kind = \"telegram\", text = \"{{steps.a.state}} {{steps.again.iterations}} {{loop.index}}\" }]\n[[workflows.steps]]\nid = \"after\"\nkind = \"telegram\"\ntext = \"{{steps.say}} {{event.service.id}} {{secrets.token}}\"\n",
    );
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn loop_values_variables_and_inputs_are_checked() {
    let found = one(
        "[[workflows.steps]]\nid = \"a\"\nkind = \"telegram\"\ntext = \"{{loop.item}}\"\n[[workflows.steps]]\nid = \"b\"\nkind = \"telegram\"\ntext = \"{{vars.later}}\"\n[[workflows.steps]]\nid = \"c\"\nkind = \"set\"\nvariable = \"later\"\nvalue = \"{{inputs.colour}}\"\n[[workflows.steps]]\nid = \"d\"\nkind = \"telegram\"\ntext = \"{{vars.later}} {{inputs.service}}\"\n",
    );
    assert_eq!(
        found,
        vec![
            "workflows[0].steps[0].text",
            "workflows[0].steps[1].text",
            "workflows[0].steps[2].value",
        ]
    );
}

#[test]
fn a_while_condition_may_read_the_loop_index() {
    let found = one(
        "[[workflows.steps]]\nid = \"poll\"\nkind = \"loop\"\nwhile = { left = \"{{loop.index}}\", op = \"<\", right = \"3\" }\nbody = [{ id = \"w\", kind = \"wait\", seconds = 1 }]\n",
    );
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn an_event_field_must_be_one_some_event_carries() {
    let found = one(
        "[[workflows.steps]]\nid = \"a\"\nkind = \"log\"\nmessage = \"{{event.name}} {{event.at}} {{event.event.name}} {{event.service.id}} {{event.run.by}} {{event.webhook.branch}} {{event.webhook.body.commits.0.id}}\"\n[[workflows.steps]]\nid = \"b\"\nkind = \"log\"\nmessage = \"{{event.servce.id}}\"\n",
    );
    assert_eq!(found, vec!["workflows[0].steps[1].message"]);
}
