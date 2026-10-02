use toml_edit::DocumentMut;

use crate::types::AutomationsSection;

#[test]
fn a_workflow_keeps_its_outputs_and_their_comments_when_written_back() {
    use super::replace_workflow;
    use crate::services::decode_workflow;
    let text = r#"# weather for the home page
[[workflows]]
id = "weather"
title = "Weather"
outputs = [{ name = "temperature", value = "{{vars.now}}", description = "°C now" }]

[[workflows.steps]]
id = "now"
kind = "set"
variable = "now"
value = "20"
"#;
    let mut document: DocumentMut = text.parse().unwrap();
    let section = AutomationsSection::read(&document).unwrap();
    let raw = &section.workflows[0];
    let mut workflow = decode_workflow(raw).unwrap();
    assert_eq!(workflow.outputs[0].description.as_deref(), Some("°C now"));
    workflow.title = "Weather now".into();
    replace_workflow(&mut document, 0, &workflow, &raw.steps);
    let written = document.to_string();
    assert!(
        written.starts_with("# weather for the home page\n"),
        "{written}"
    );
    assert!(
        written.contains("outputs = [{ name = \"temperature\", value = \"{{vars.now}}\", description = \"°C now\" }]"),
        "{written}"
    );
    let again = AutomationsSection::read(&document).unwrap();
    assert_eq!(
        decode_workflow(&again.workflows[0]).unwrap().outputs,
        workflow.outputs
    );
}
