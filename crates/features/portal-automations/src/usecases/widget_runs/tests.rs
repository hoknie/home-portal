use std::collections::BTreeMap;
use std::time::Duration;

use serde_json::{Value, json};

use crate::features::tests::{eventually, portal, start};
use crate::types::{Outcome, RunFilter, SourceCall, ValueType, WidgetSupport};

const ENGINE: &str = r#"[modules]
workflows = true

[[workflows]]
id = "disks"
title = "Disks"

[[workflows.steps]]
id = "free"
kind = "set"
variable = "free"
json = "120"

[[workflows]]
id = "weather"
title = "Weather"
outputs = [
  { name = "temperature", value = "{{vars.temperature}}", description = "°C now" },
  { name = "current", value = "{{steps.open_meteo.json.current}}" },
]

[[workflows.steps]]
id = "measure"
kind = "set"
variable = "temperature"
json = "20"

[[workflows.steps]]
id = "skip"
kind = "if"
condition = { left = "{{vars.temperature}}", op = "==", right = "-100" }
then = [{ id = "open_meteo", kind = "http", method = "GET", url = "https://api.open-meteo.com/v1/forecast", response_sample = '{"current": {"temperature_2m": 20}}' }]

[[workflows]]
id = "aimed"
title = "Aimed"
inputs = ["target"]

[[workflows.steps]]
id = "keep"
kind = "set"
variable = "target"
value = "{{inputs.target}}"

[[workflows]]
id = "broken"
title = "Broken"

[[workflows.steps]]
id = "fail"
kind = "stop"
outcome = "failed"
reason = "nas unreachable"

[[automations]]
id = "restart-media"
title = "Restart Jellyfin"
when = { event = "manual" }
run = { script = "restart.sh" }
"#;

const SCRIPTS: [(&str, &str); 3] = [
    ("restart.sh", "echo restarted"),
    (
        "clients.sh",
        "printf '{\"clients\": [{\"name\": \"phone\"}]}'",
    ),
    ("backup.sh", "echo 'backup ok, 3 h ago'"),
];

fn engine() -> (tempfile::TempDir, crate::AutomationsFeature, WidgetSupport) {
    let (folder, feature) = portal(ENGINE, &SCRIPTS);
    let support = feature.widget_support();
    (folder, feature, support)
}

fn workflow(id: &str) -> SourceCall {
    SourceCall::Workflow {
        id: id.to_string(),
        inputs: BTreeMap::new(),
    }
}

fn script(name: &str) -> SourceCall {
    SourceCall::Script {
        script: name.to_string(),
        args: Vec::new(),
        timeout: Duration::from_secs(5),
    }
}

#[tokio::test]
async fn a_source_gives_the_vars_or_the_declared_outputs_of_a_workflow_and_the_output_of_a_script()
{
    let (_folder, _feature, support) = engine();
    assert_eq!(
        support
            .run_source
            .run("disks", "Disks", &workflow("disks"))
            .await,
        Ok(json!({ "free": 120 }))
    );
    assert_eq!(
        support.run_source.run("w", "W", &workflow("weather")).await,
        Ok(json!({ "temperature": 20, "current": null }))
    );
    assert_eq!(
        support
            .run_source
            .run("c", "C", &script("clients.sh"))
            .await
            .unwrap()["clients"][0]["name"],
        "phone"
    );
    assert_eq!(
        support.run_source.run("b", "B", &script("backup.sh")).await,
        Ok(Value::String("backup ok, 3 h ago".into()))
    );
}

#[tokio::test]
async fn a_failing_or_incomplete_source_is_named_in_general_words_and_journaled_once() {
    let (_folder, feature, support) = engine();
    assert_eq!(
        support
            .run_source
            .run("failing", "F", &workflow("broken"))
            .await,
        Err("the workflow failed".into())
    );
    assert_eq!(
        support
            .run_source
            .run("needs", "N", &workflow("aimed"))
            .await,
        Err("the input target is not given".into())
    );
    let journal = feature.state.sink.journal.clone();
    assert_eq!(
        journal
            .matching(&RunFilter {
                widget: Some("failing".into()),
                ..Default::default()
            })
            .len(),
        1
    );
    support
        .run_source
        .run("disks", "Disks", &workflow("disks"))
        .await
        .unwrap();
    assert!(
        journal
            .matching(&RunFilter {
                widget: Some("disks".into()),
                ..Default::default()
            })
            .is_empty()
    );
    assert_eq!(support.run_source.available_permits(), 4);
}

#[tokio::test]
async fn a_button_starts_its_automation_or_workflow_as_the_person_and_both_run() {
    let (_folder, feature, support) = engine();
    start(&feature);
    let automation = support
        .start_automation
        .run("disks", Some("Disks"), "restart-media", &[], "anna")
        .unwrap();
    let workflow = support
        .start_workflow
        .run("weather", None, "disks", Vec::new(), "anna")
        .unwrap();
    let journal = feature.state.sink.journal.clone();
    for (run, widget) in [(automation, "widget:disks"), (workflow, "widget:weather")] {
        assert!(
            eventually(|| journal
                .find(run)
                .is_some_and(|record| record.result.outcome == Outcome::Succeeded))
            .await,
            "{:?}",
            journal.find(run)
        );
        let record = journal.find(run).unwrap();
        assert_eq!(record.automation, widget);
        assert!(
            record
                .fields
                .iter()
                .any(|(key, value)| key == "run.by" && value == "anna")
        );
    }
    assert!(
        support
            .start_automation
            .run("disks", None, "nope", &[], "anna")
            .is_err()
    );
}

#[test]
fn references_and_templates_are_checked_against_the_configuration() {
    let (_folder, _feature, support) = engine();
    assert!(
        support
            .check_call
            .run("nope", &BTreeMap::new())
            .iter()
            .any(|error| error.field == "workflow")
    );
    let unknown = BTreeMap::from([("ghost".to_string(), Some(json!("x")))]);
    assert!(!support.check_call.run("aimed", &unknown).is_empty());
    assert!(support.check_script.run("../secret.sh").is_some());
    assert!(support.check_script.run("backup.sh").is_none());
    assert_eq!(
        support
            .event_fields
            .run("restart-media")
            .map(|(event, _)| event),
        Some("manual".to_string())
    );
    let paths: Vec<String> = support
        .declared_paths
        .run("weather")
        .into_iter()
        .map(|path| path.path)
        .collect();
    assert_eq!(
        paths,
        [
            "data.temperature",
            "data.current",
            "data.current.temperature_2m"
        ]
    );
    assert_eq!(
        support.source_timeout.run("disks"),
        Duration::from_secs(300)
    );
    let allows = |name: &str| {
        if name.starts_with("data") {
            Ok(ValueType::Any)
        } else {
            Err(format!("names {name}"))
        }
    };
    assert_eq!(
        support.check_template.run("{{data.free | round}}", &allows),
        None
    );
    assert_eq!(
        support.check_template.run("{{steps.x}}", &allows),
        Some("names steps.x".into())
    );
    let mut templates = support.open_templates.run(
        &json!({ "free": 120, "rows": [1, 2] }),
        "disks",
        Some("Disks"),
        None,
    );
    assert_eq!(
        templates.text("{{data.free}} GB in {{widget.title}}"),
        Ok("120 GB in Disks".into())
    );
    assert_eq!(templates.value("{{data.rows}}"), Ok(json!([1, 2])));
    templates.set_item(Some((json!({ "name": "tv" }), 1)));
    assert_eq!(templates.text("{{index}} {{item.name}}"), Ok("1 tv".into()));
}
