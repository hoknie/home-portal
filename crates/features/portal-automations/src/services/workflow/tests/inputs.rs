use serde_json::json;

use super::running::run;
use super::support::fields;
use crate::services::workflow::workflow_errors;
use crate::types::Ending;

const TYPED: &str = "[[workflows]]\nid = \"w\"\ntitle = \"W\"\ninputs = [\"service\", { name = \"hosts\", type = \"list\", default = [\"nas\", \"router\"] }, { name = \"retries\", type = \"number\", default = 2, description = \"How many times\" }]\n";

#[test]
fn typed_inputs_load_beside_plain_names() {
    let section = super::support::section(&format!(
        "{TYPED}[[workflows.steps]]\nid = \"say\"\nkind = \"notify\"\ntext = \"{{{{inputs.hosts.1}}}} {{{{inputs.retries | round}}}}\"\n"
    ));
    assert!(
        workflow_errors(&section).is_empty(),
        "{:?}",
        workflow_errors(&section)
    );
    let workflow = &crate::services::decoded_workflows(&section)[0];
    let names: Vec<(&str, &str)> = workflow
        .inputs
        .iter()
        .map(|input| (input.name.as_str(), input.input_type.name()))
        .collect();
    assert_eq!(
        names,
        vec![
            ("service", "text"),
            ("hosts", "list"),
            ("retries", "number")
        ]
    );
    assert_eq!(
        workflow.inputs[2].description.as_deref(),
        Some("How many times")
    );
}

#[test]
fn a_default_of_the_wrong_type_an_unknown_type_and_a_repeated_name_are_refused() {
    let found = fields(
        "[[workflows]]\nid = \"w\"\ntitle = \"W\"\ninputs = [{ name = \"retries\", type = \"number\", default = \"three\" }, { name = \"x\", type = \"date\" }, \"retries\"]\n[[workflows.steps]]\nid = \"n\"\nkind = \"nothing\"\n",
    );
    assert_eq!(
        found,
        vec![
            "workflows[0].inputs[0].default",
            "workflows[0].inputs[1].type",
            "workflows[0].inputs[2]",
        ]
    );
}

#[test]
fn a_text_filter_on_a_number_input_is_refused_at_load() {
    let found = fields(&format!(
        "{TYPED}[[workflows.steps]]\nid = \"say\"\nkind = \"notify\"\ntext = \"{{{{inputs.retries | upper}}}}\"\n"
    ));
    assert_eq!(found, vec!["workflows[0].steps[0].text"]);
}

#[tokio::test]
async fn defaults_fill_missing_inputs_and_a_list_is_walked_by_index() {
    let outcome = run(&format!(
        "{TYPED}[[workflows.steps]]\nid = \"pick\"\nkind = \"set\"\nvariable = \"second\"\nvalue = \"{{{{inputs.hosts.1}}}}\"\n[[workflows.steps]]\nid = \"all\"\nkind = \"set\"\nvariable = \"hosts\"\njson = \"{{{{inputs.hosts}}}}\"\n"
    ))
    .await;
    assert_eq!(outcome.ending, Ending::Succeeded(None));
    assert_eq!(outcome.frame.vars["second"], json!("router"));
    assert_eq!(outcome.frame.vars["hosts"], json!(["nas", "router"]));
    assert_eq!(outcome.frame.inputs["retries"], json!(2));
}

#[tokio::test]
async fn a_called_workflow_gets_a_list_through_a_template() {
    let outcome = run("[[workflows]]\nid = \"w\"\ntitle = \"W\"\n[[workflows.steps]]\nid = \"list\"\nkind = \"set\"\nvariable = \"hosts\"\njson = '[\"a\", \"b\"]'\n[[workflows.steps]]\nid = \"call\"\nkind = \"workflow\"\nworkflow = \"count\"\ninputs = { hosts = \"{{vars.hosts}}\" }\n[[workflows]]\nid = \"count\"\ntitle = \"Count\"\ninputs = [{ name = \"hosts\", type = \"list\" }]\n[[workflows.steps]]\nid = \"n\"\nkind = \"set\"\nvariable = \"n\"\nvalue = \"{{inputs.hosts | length}}\"\n").await;
    assert_eq!(outcome.ending, Ending::Succeeded(None));
    assert_eq!(outcome.frame.steps["call"]["vars"]["n"], json!(2));
}

#[tokio::test]
async fn a_value_that_does_not_read_as_its_type_fails_the_call() {
    let outcome = run("[[workflows]]\nid = \"w\"\ntitle = \"W\"\n[[workflows.steps]]\nid = \"call\"\nkind = \"workflow\"\nworkflow = \"count\"\ninputs = { hosts = \"not a list\" }\n[[workflows]]\nid = \"count\"\ntitle = \"Count\"\ninputs = [{ name = \"hosts\", type = \"list\" }]\n[[workflows.steps]]\nid = \"n\"\nkind = \"nothing\"\n").await;
    let Ending::Failed(reason) = outcome.ending else {
        panic!("{:?}", outcome.ending);
    };
    assert!(
        reason.contains("the input hosts must be a list"),
        "{reason}"
    );
}

#[tokio::test]
async fn a_list_and_a_dictionary_are_built_from_rows_keeping_each_value_type() {
    let outcome = run("[[workflows]]\nid = \"w\"\ntitle = \"W\"\ninputs = [\"extra\"]\n[[workflows.steps]]\nid = \"count\"\nkind = \"set\"\nvariable = \"count\"\njson = \"3\"\n[[workflows.steps]]\nid = \"hosts\"\nkind = \"set\"\nvariable = \"hosts\"\nlist = [\"nas\", \"{{inputs.extra}}\", \"{{vars.count}}\"]\n[[workflows.steps]]\nid = \"report\"\nkind = \"set\"\nvariable = \"report\"\nobject = { host = \"{{vars.hosts.0}}\", count = \"{{vars.count}}\", note = \"n: {{vars.count}}\" }\n[[workflows.steps]]\nid = \"say\"\nkind = \"set\"\nvariable = \"host\"\nvalue = \"{{vars.report.host}}\"\n").await;
    assert_eq!(outcome.ending, Ending::Succeeded(None));
    assert_eq!(outcome.frame.vars["hosts"], json!(["nas", null, 3]));
    assert_eq!(
        outcome.frame.vars["report"],
        json!({"host": "nas", "count": 3, "note": "n: 3"})
    );
    assert_eq!(outcome.frame.vars["host"], json!("nas"));
}

#[test]
fn a_set_step_needs_exactly_one_of_its_four_forms() {
    let found = fields(
        "[[workflows]]\nid = \"w\"\ntitle = \"W\"\n[[workflows.steps]]\nid = \"both\"\nkind = \"set\"\nvariable = \"x\"\nvalue = \"a\"\nlist = [\"b\"]\n[[workflows.steps]]\nid = \"none\"\nkind = \"set\"\nvariable = \"y\"\n",
    );
    assert_eq!(
        found,
        vec!["workflows[0].steps[0]", "workflows[0].steps[1]"]
    );
    let scoped = fields(
        "[[workflows]]\nid = \"w\"\ntitle = \"W\"\n[[workflows.steps]]\nid = \"bad\"\nkind = \"set\"\nvariable = \"z\"\nobject = { k = \"{{vars.missing}}\" }\n",
    );
    assert_eq!(scoped, vec!["workflows[0].steps[0].object.k"]);
}

#[tokio::test]
async fn an_automation_step_passes_rendered_fields_and_its_chain() {
    let outcome = run("[[workflows]]\nid = \"w\"\ntitle = \"W\"\ninputs = [\"service\"]\n[[workflows.steps]]\nid = \"go\"\nkind = \"automation\"\nautomation = \"restart-media\"\nfields = { \"service.id\" = \"{{inputs.service}}\" }\nwait = true\n[[workflows.steps]]\nid = \"back\"\nkind = \"automation\"\nautomation = \"nas-down\"\n[[automations]]\nid = \"restart-media\"\ntitle = \"Restart\"\nwhen = { event = \"service.status-changed\" }\nrun = { script = \"restart.sh\" }\n[[automations]]\nid = \"nas-down\"\ntitle = \"NAS down\"\nwhen = { event = \"manual\" }\nworkflow = \"w\"\ninputs = { service = \"nas\" }\n").await;
    let Ending::Failed(reason) = &outcome.ending else {
        panic!("{:?}", outcome.ending);
    };
    assert!(reason.contains("already started this run"), "{reason}");
    assert_eq!(
        outcome.frame.steps["go"],
        json!({"run_id": "1", "outcome": "succeeded"})
    );
    let started = outcome.starter.started.lock().unwrap();
    assert_eq!(
        *started,
        vec![(
            "restart-media".to_string(),
            vec![("service.id".to_string(), "nas".to_string())],
            vec!["nas-down".to_string()]
        )]
    );
}

#[test]
fn an_automation_step_names_a_known_automation_and_fields_of_its_event() {
    let found = fields(
        "[[workflows]]\nid = \"w\"\ntitle = \"W\"\n[[workflows.steps]]\nid = \"go\"\nkind = \"automation\"\nautomation = \"restart\"\nfields = { \"user.name\" = \"x\", \"service.id\" = \"nas\" }\n[[workflows.steps]]\nid = \"lost\"\nkind = \"automation\"\nautomation = \"nowhere\"\n[[automations]]\nid = \"restart\"\ntitle = \"Restart\"\nwhen = { event = \"service.status-changed\" }\nrun = { script = \"restart.sh\" }\n",
    );
    assert_eq!(
        found,
        vec![
            "workflows[0].steps[0].fields.user.name",
            "workflows[0].steps[1].automation",
        ]
    );
}

const WEATHER: &str = "[[workflows]]\nid = \"weather\"\ntitle = \"Weather\"\n[[workflows.steps]]\nid = \"open_meteo\"\nkind = \"set\"\nvariable = \"answer\"\njson = '{\"current\": {\"temperature_2m\": 20, \"wind_speed_10m\": 10.8}}'\n[[workflows.steps]]\nid = \"read\"\nkind = \"set\"\nvariable = \"wind\"\nvalue = \"{{vars.answer.current.wind_speed_10m}}\"\n";

fn with_outputs(outputs: &str) -> String {
    format!("{WEATHER}[[workflows.outputs]]\n{outputs}")
}

#[tokio::test]
async fn outputs_are_rendered_in_order_when_a_run_succeeds_keeping_their_type() {
    let text = format!(
        "{}[[workflows.outputs]]\nname = \"wind\"\nvalue = \"{{{{vars.wind}}}} km/h\"\n",
        with_outputs(
            "name = \"temperature\"\nvalue = \"{{vars.answer.current.temperature_2m}}\"\ndescription = \"°C now\"\n"
        )
    );
    assert!(workflow_errors(&super::support::section(&text)).is_empty());
    let outcome = run(&text).await;
    assert_eq!(outcome.ending, Ending::Succeeded(None));
    let outputs = outcome.frame.outputs.expect("outputs");
    assert_eq!(outputs["temperature"], json!(20));
    assert_eq!(outputs["wind"], json!("10.8 km/h"));
}

#[tokio::test]
async fn a_run_that_fails_has_no_outputs() {
    let text = format!(
        "{WEATHER}[[workflows.steps]]\nid = \"end\"\nkind = \"stop\"\noutcome = \"failed\"\n{}",
        "[[workflows.outputs]]\nname = \"wind\"\nvalue = \"{{vars.wind}}\"\n"
    );
    let outcome = run(&text).await;
    assert!(
        matches!(outcome.ending, Ending::Failed(_)),
        "{:?}",
        outcome.ending
    );
    assert!(outcome.frame.outputs.is_none());
}

#[test]
fn an_output_names_only_what_the_end_of_a_workflow_knows() {
    let errors = workflow_errors(&super::support::section(&with_outputs(
        "name = \"item\"\nvalue = \"{{loop.item}}\"\n[[workflows.outputs]]\nname = \"who\"\nvalue = \"{{event.service.id}}\"\n[[workflows.outputs]]\nname = \"gone\"\nvalue = \"{{vars.missing}}\"\n",
    )));
    let named: Vec<(&str, &str)> = errors
        .iter()
        .map(|error| (error.field.as_str(), error.message.as_str()))
        .collect();
    assert!(
        named
            .iter()
            .any(|(field, message)| *field == "workflows[0].outputs[0].value"
                && message.contains("only inside a loop")),
        "{named:?}"
    );
    assert!(
        named
            .iter()
            .any(|(field, message)| *field == "workflows[0].outputs[1].value"
                && message.contains("not the event")),
        "{named:?}"
    );
    assert!(
        named
            .iter()
            .any(|(field, _)| *field == "workflows[0].outputs[2].value"),
        "{named:?}"
    );
    let twice = workflow_errors(&super::support::section(&with_outputs(
        "name = \"wind\"\nvalue = \"{{vars.wind}}\"\n[[workflows.outputs]]\nname = \"wind\"\nvalue = \"x\"\n",
    )));
    assert!(
        twice
            .iter()
            .any(|error| error.field == "workflows[0].outputs[1].name"
                && error.message.contains("another output")),
        "{twice:?}"
    );
}

#[tokio::test]
async fn a_secret_in_an_output_is_masked() {
    use std::sync::Arc;
    let lookup: crate::services::workflow::SecretLookup =
        Arc::new(|key: &str| (key == "token").then(|| "abc123".to_string()));
    let (_sender, stop) = tokio::sync::watch::channel(false);
    let outcome = super::running::run_prepared(
        &with_outputs("name = \"key\"\nvalue = \"{{secrets.token}}\"\n"),
        (Arc::new(super::running::FakeActions::default()), stop),
        &[],
        crate::services::workflow::Secrets::new(lookup),
    )
    .await;
    assert_eq!(outcome.frame.outputs.expect("outputs")["key"], json!("***"));
}
