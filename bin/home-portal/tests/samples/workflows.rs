use portal_automations::{
    InputResponse, QueuedResponse, WorkflowCatalogue, WorkflowCatalogueResponse, WorkflowResponse,
    WorkflowUsageResponse, WorkflowsResponse,
};
use serde_json::json;

use crate::automations::{outcome, run, traced};
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
