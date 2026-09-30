use portal_feature::FieldError;
use toml_edit::DocumentMut;

use crate::services::validate_automations;

const CAMERA: &str = "7d3f2a4e-5b1c-4e8f-9a2d-6c0b1e3f4a5d";
const DEPLOY: &str = "0b9e8c2a-1d3f-4a5b-8c7d-9e0f1a2b3c4d";

fn errors(text: &str) -> Vec<FieldError> {
    validate_automations(&text.parse::<DocumentMut>().unwrap())
}

fn fields(text: &str) -> Vec<String> {
    errors(text).into_iter().map(|error| error.field).collect()
}

fn workflow() -> &'static str {
    "[[workflows]]\nid = \"alarm\"\ntitle = \"Alarm\"\ninputs = [\"zone\", \"camera\"]\n\n[[workflows.steps]]\nid = \"done\"\nkind = \"stop\"\noutcome = \"succeeded\"\n\n"
}

fn webhooks() -> String {
    format!(
        "[[webhooks]]\nid = \"{CAMERA}\"\ntitle = \"Camera\"\nvariables = [\"camera\"]\naction = \"event\"\n\n[[webhooks]]\nid = \"{DEPLOY}\"\ntitle = \"Deploy\"\nvariables = [\"branch\"]\naction = \"script\"\nrun = {{ script = \"a.sh\" }}\n\n"
    )
}

#[test]
fn an_undeclared_variable_in_a_workflow_input_is_refused() {
    let text = format!(
        "{}{}[[automations]]\nid = \"a\"\ntitle = \"A\"\nwhen = {{ event = \"webhook.received\", webhooks = [\"{CAMERA}\"] }}\nworkflow = \"alarm\"\ninputs = {{ zone = \"{{{{webhook.zone}}}}\", camera = \"{{{{webhook.camera}}}}\" }}\n",
        workflow(),
        webhooks()
    );
    assert_eq!(fields(&text), vec!["automations[0].inputs.zone"]);
    assert!(errors(&text)[0].message.contains("webhook.zone"));
}

#[test]
fn an_unfiltered_automation_needs_the_variable_only_from_event_webhooks() {
    let text = format!(
        "{}[[automations]]\nid = \"a\"\ntitle = \"A\"\nwhen = {{ event = \"webhook.received\" }}\nrun = {{ script = \"a.sh\", args = [\"{{{{webhook.camera}}}}\"] }}\n",
        webhooks()
    );
    assert!(errors(&text).is_empty(), "{:?}", errors(&text));
}

#[test]
fn the_body_is_always_there() {
    let text = format!(
        "{}[[automations]]\nid = \"a\"\ntitle = \"A\"\nwhen = {{ event = \"webhook.received\" }}\nrun = {{ script = \"a.sh\", args = [\"{{{{webhook.body.action}}}}\", \"{{{{webhook.body.commits.0.id}}}}\", \"{{{{webhook.body}}}}\"] }}\n",
        webhooks()
    );
    assert!(errors(&text).is_empty(), "{:?}", errors(&text));
}

#[test]
fn a_webhook_workflow_input_that_names_an_undeclared_variable_is_refused() {
    let text = format!(
        "{}[[webhooks]]\nid = \"{CAMERA}\"\ntitle = \"Deploy\"\nvariables = [\"branch\"]\naction = \"script\"\nworkflow = \"alarm\"\ninputs = {{ zone = \"{{{{webhook.brnch}}}}\" }}\n",
        workflow()
    );
    assert_eq!(fields(&text), vec!["webhooks[0].inputs.zone"]);
    assert!(errors(&text)[0].message.contains("webhook.brnch"));
}

#[test]
fn a_reserved_variable_name_is_refused() {
    let text = format!(
        "[[webhooks]]\nid = \"{CAMERA}\"\ntitle = \"Camera\"\nvariables = [\"body\"]\naction = \"event\"\n"
    );
    let found = errors(&text);
    assert_eq!(
        found
            .iter()
            .map(|error| error.field.as_str())
            .collect::<Vec<_>>(),
        vec!["webhooks[0].variables[0]"]
    );
    assert!(found[0].message.contains("id, title, body"));
}
