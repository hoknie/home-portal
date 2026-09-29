use portal_automations::validate_automations;
use portal_automations::{
    InputResponse, PortalService, PortalState, QueuedResponse, RenderedResponse,
    TraceEntryResponse, TraceResponse, WorkflowCatalogue, WorkflowCatalogueResponse,
    WorkflowResponse, WorkflowUsageResponse, WorkflowsResponse,
};
use serde_json::{Map, Value, json};
use toml_edit::{Array, DocumentMut, InlineTable};

use crate::automations::{outcome, run};
use crate::check;

fn revive() -> WorkflowResponse {
    let mut last = run(
        "12",
        "nas-down",
        "service.status-changed",
        outcome("succeeded", None, Some("nas is back")),
    );
    last.workflow = Some("revive".into());
    last.trace = Some(traced(&[
        (
            "steps[0]",
            "first_probe",
            "probe",
            None,
            "succeeded",
            "down",
        ),
        ("steps[1]", "down", "if", None, "succeeded", "then"),
    ]));
    WorkflowResponse {
        steps_version: "5d41402abc4b".into(),
        id: "revive".into(),
        title: "Revive a service".into(),
        enabled: true,
        description: Some("Probe, retry the restart API three times, then report".into()),
        tags: vec!["media".into()],
        timeout_seconds: 600,
        inputs: vec![
            InputResponse {
                name: "service".into(),
                input_type: "text".into(),
                default: None,
                description: None,
            },
            InputResponse {
                name: "tries".into(),
                input_type: "number".into(),
                default: Some(json!(3)),
                description: Some("How many restarts to try".into()),
            },
        ],
        steps: json!([
            {"id": "first_probe", "label": "Probe it now", "kind": "probe", "service": "{{inputs.service}}"},
            {
                "id": "down",
                "kind": "if",
                "condition": {"left": "{{steps.first_probe.state}}", "op": "!=", "right": "up"},
                "then": [
                    {
                        "id": "retry",
                        "kind": "loop",
                        "repeat": 3,
                        "body": [
                            {"id": "restart", "kind": "http", "method": "POST", "url": "http://192.168.1.10:9000/restart/{{inputs.service}}", "fail_on_error": false},
                            {"id": "settle", "kind": "wait", "seconds": 20}
                        ]
                    }
                ]
            },
            {
                "id": "both",
                "kind": "parallel",
                "branches": [
                    [{"id": "nas_state", "kind": "status", "service": "nas"}],
                    [{"id": "router_state", "kind": "status", "service": "router"}]
                ]
            },
            {"id": "tell", "kind": "telegram", "text": "{{inputs.service}} is {{steps.nas_state.state}}"}
        ]),
        used_by: vec![WorkflowUsageResponse {
            kind: "automation".into(),
            id: "nas-down".into(),
            title: "NAS down".into(),
        }],
        last_run: Some(last),
        active_run: None,
    }
}

fn spare() -> WorkflowResponse {
    WorkflowResponse {
        steps_version: "7d793037a076".into(),
        id: "note".into(),
        title: "Note".into(),
        enabled: false,
        description: None,
        tags: Vec::new(),
        timeout_seconds: 300,
        inputs: Vec::new(),
        steps: json!([
            {"id": "remember", "kind": "set", "variable": "note", "value": "hello"},
            {"id": "done", "kind": "stop", "outcome": "succeeded", "reason": "{{vars.note}}"}
        ]),
        used_by: Vec::new(),
        last_run: None,
        active_run: None,
    }
}

#[test]
fn the_workflow_samples_match_their_serializers() {
    let listed = WorkflowsResponse {
        workflows: vec![revive(), spare()],
    };
    check("workflows", serde_json::to_value(&listed).unwrap());
    check(
        "workflow-catalogue",
        serde_json::to_value(WorkflowCatalogueResponse::of(&WorkflowCatalogue.run())).unwrap(),
    );
    check(
        "workflow-run",
        serde_json::to_value(QueuedResponse {
            run_id: "13".into(),
        })
        .unwrap(),
    );
}

#[test]
fn the_portal_values_sample_matches_its_shape() {
    let service = |id: &str, name: &str, state: &str| PortalService {
        id: id.into(),
        name: name.into(),
        group: Some("Home".into()),
        url: format!("http://{id}.lan"),
        address: format!("http://192.168.1.10/{id}"),
        state: state.into(),
        since: Some("2026-09-28T09:00:00Z".into()),
        latency_milliseconds: Some(12),
        public: id == "media",
    };
    let state = PortalState {
        services: vec![
            service("media", "Media", "up"),
            service("nas", "NAS", "down"),
        ],
        address: "0.0.0.0".into(),
        port: 8080,
        url: "http://portal.lan:8080".into(),
        environments: vec!["local".into(), "vpn".into(), "internet".into()],
    };
    check(
        "workflow-portal",
        state.value(&[("proxy", false), ("users", true), ("workflows", true)]),
    );
}

fn nothing(id: &str) -> Value {
    json!({"id": id, "kind": "nothing"})
}

fn defaults() -> Vec<Value> {
    vec![
        json!({"id": "check", "kind": "if", "condition": {"left": "a", "op": "==", "right": "a"}, "then": [nothing("then_step")]}),
        json!({"id": "again", "kind": "loop", "repeat": 1, "body": [nothing("body_step")]}),
        json!({"id": "both", "kind": "parallel", "branches": [[nothing("left_step")], [nothing("right_step")]]}),
        json!({"id": "call", "kind": "workflow", "workflow": "other"}),
        nothing("idle"),
        json!({"id": "leave", "kind": "break"}),
        json!({"id": "skip", "kind": "continue"}),
        json!({"id": "done", "kind": "stop", "outcome": "succeeded"}),
        json!({"id": "remember", "kind": "set", "variable": "note", "value": "x"}),
        json!({"id": "pause", "kind": "wait", "seconds": 1}),
        json!({"id": "reshape", "kind": "transform", "input": "[]", "operations": [{"op": "reverse"}]}),
        json!({"id": "ask", "kind": "http", "method": "GET", "url": "http://nas.lan"}),
        json!({"id": "run", "kind": "script", "script": "restart.sh", "timeout_seconds": 60}),
        json!({"id": "tell", "kind": "notify", "text": "hello"}),
        json!({"id": "note", "kind": "log", "message": "x", "level": "info"}),
        json!({"id": "start", "kind": "automation", "automation": "manual-one"}),
        json!({"id": "probe_it", "kind": "probe", "service": "nas"}),
        json!({"id": "status_of", "kind": "status", "service": "nas"}),
    ]
}

fn toml_value(value: &Value) -> toml_edit::Value {
    match value {
        Value::String(text) => text.as_str().into(),
        Value::Bool(flag) => (*flag).into(),
        Value::Number(number) => number.as_i64().unwrap_or_default().into(),
        Value::Array(items) => {
            toml_edit::Value::Array(items.iter().map(toml_value).collect::<Array>())
        }
        Value::Object(map) => {
            let mut table = InlineTable::new();
            for (key, item) in map {
                table.insert(key, toml_value(item));
            }
            toml_edit::Value::InlineTable(table)
        }
        Value::Null => "".into(),
    }
}

fn inside_a_loop_when_it_needs_one(step: &Value) -> Value {
    match step["kind"].as_str() {
        Some("break" | "continue") => json!({
            "id": format!("{}_loop", step["id"].as_str().unwrap_or_default()),
            "kind": "loop",
            "repeat": 1,
            "body": [step],
        }),
        _ => step.clone(),
    }
}

#[test]
fn every_step_kind_has_a_minimal_valid_step_shared_with_the_interface() {
    let steps = defaults();
    let catalogue = WorkflowCatalogueResponse::of(&WorkflowCatalogue.run());
    let kinds: Vec<String> = catalogue
        .kinds
        .iter()
        .map(|kind| kind.name.clone())
        .collect();
    let covered: Vec<String> = steps
        .iter()
        .map(|step| step["kind"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        covered, kinds,
        "every kind of the catalogue needs a minimal step here, in catalogue order"
    );
    let steps_text: Vec<String> = steps
        .iter()
        .map(|step| toml_value(&inside_a_loop_when_it_needs_one(step)).to_string())
        .collect();
    let text = format!(
        "[modules]\nworkflows = true\n\n[[workflows]]\nid = \"all\"\ntitle = \"All\"\nsteps = [{}]\n\n[[workflows]]\nid = \"other\"\ntitle = \"Other\"\nsteps = [{{ id = \"n\", kind = \"nothing\" }}]\n\n[[automations]]\nid = \"manual-one\"\ntitle = \"Manual\"\nwhen = {{ event = \"manual\" }}\nrun = {{ script = \"restart.sh\" }}\n",
        steps_text.join(", ")
    );
    let document: DocumentMut = text.parse().unwrap();
    assert_eq!(validate_automations(&document), Vec::new());
    let by_kind: Map<String, Value> = steps
        .into_iter()
        .map(|step| (step["kind"].as_str().unwrap().to_string(), step))
        .collect();
    check("step-defaults", Value::Object(by_kind));
}

pub type Entry<'a> = (&'a str, &'a str, &'a str, Option<usize>, &'a str, &'a str);

pub fn traced(entries: &[Entry]) -> TraceResponse {
    TraceResponse {
        entries: entries
            .iter()
            .map(
                |(path, step, kind, iteration, outcome, detail)| TraceEntryResponse {
                    path: path.to_string(),
                    step: step.to_string(),
                    label: step.to_string(),
                    kind: kind.to_string(),
                    iteration: *iteration,
                    outcome: outcome.to_string(),
                    started_at: "2026-09-25T03:00:00Z".into(),
                    duration_milliseconds: 120,
                    detail: detail.to_string(),
                    output: (*kind == "http").then(|| "{\"state\":\"down\"}".to_string()),
                    shape: (*kind == "http").then(|| "{\"state\":\"down\"}".to_string()),
                    values: (*kind == "if")
                        .then(|| RenderedResponse {
                            template: "{{steps.first_probe.state}}".into(),
                            value: "\"down\"".into(),
                        })
                        .into_iter()
                        .collect(),
                    log: match *kind {
                        "if" => vec!["\"down\" equals \"down\": yes".into(), "took then".into()],
                        "log" => vec!["[warning] nas is down".into()],
                        _ => Vec::new(),
                    },
                    values_dropped: 0,
                    log_dropped: 0,
                    item: iteration.map(|index| format!("\"try {}\"", index + 1)),
                    level: (*kind == "log").then(|| "warning".to_string()),
                },
            )
            .collect(),
        dropped: 0,
    }
}
