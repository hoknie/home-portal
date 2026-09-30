use axum::body::Body;
use axum::http::header::CONTENT_TYPE;
use axum::http::{Request, StatusCode};
use serde_json::Value;

use crate::controllers::tests::automations::{Api, api, get, send};
use crate::features::tests::{eventually, start, write_script};

pub const CAMERA: &str = "9a8b7c6d-5e4f-4a3b-8c2d-1e0f9a8b7c6d";
pub const DEPLOY: &str = "1f2e3d4c-5b6a-4978-8a6b-5c4d3e2f1a0b";
pub const ECHO: &str = "2a3b4c5d-6e7f-4a8b-9c0d-1e2f3a4b5c6d";
pub const INPUT: &str = "5d6e7f8a-9b0c-4d1e-8f2a-3b4c5d6e7f8a";
pub const ALARM: &str = "6e7f8a9b-0c1d-4e2f-9a3b-4c5d6e7f8a9b";
pub const SYNC: &str = "7f8a9b0c-1d2e-4f3a-8b4c-5d6e7f8a9b0c";
pub const SAFE_SYNC: &str = "8a9b0c1d-2e3f-4a4b-9c5d-6e7f8a9b0c1d";

pub fn file() -> String {
    format!(
        r#"[modules]
workflows = true

[[workflows]]
id = "alarm"
title = "Alarm"
inputs = ["camera", {{ name = "zones", type = "list" }}, {{ name = "payload", type = "object" }}]

[[workflows.steps]]
id = "done"
kind = "stop"
outcome = "succeeded"
reason = "{{{{inputs.camera}}}} {{{{inputs.zones}}}}"

[[webhooks]]
id = "{CAMERA}"
title = "Camera"
variables = ["camera"]
action = "event"

[[workflows]]
id = "sizes"
title = "Sizes"
inputs = [{{ name = "payload", type = "object" }}]

[[workflows.steps]]
id = "done"
kind = "stop"
outcome = "succeeded"
reason = "{{{{inputs.payload.tail}}}} {{{{event.webhook.body.tail}}}}"

[[workflows]]
id = "peek"
title = "Peek"

[[workflows.steps]]
id = "done"
kind = "stop"
outcome = "succeeded"
reason = "{{{{event.webhook.camera}}}} {{{{event.webhook.body.camera}}}}"

[[automations]]
id = "peek"
title = "Peek"
when = {{ event = "webhook.received", webhooks = ["{CAMERA}"] }}
workflow = "peek"

[[automations]]
id = "motion"
title = "Motion"
when = {{ event = "webhook.received", webhooks = ["{CAMERA}"] }}
workflow = "alarm"
inputs = {{ camera = "{{{{webhook.camera}}}}", zones = "{{{{webhook.body.zones}}}}" }}

[[webhooks]]
id = "{ECHO}"
title = "Echo"
action = "script"
run = {{ script = "restart.sh", args = ["{{{{webhook.body.repository.name}}}}", "{{{{webhook.body}}}}"] }}

[[webhooks]]
id = "{INPUT}"
title = "Input"
action = "script"
run = {{ script = "input.sh" }}

[[webhooks]]
id = "{ALARM}"
title = "Alarm"
action = "script"
workflow = "sizes"
inputs = {{ payload = "{{{{webhook.body}}}}" }}

[[webhooks]]
id = "{SYNC}"
title = "Sync"
variables = ["host"]
action = "script"
run = {{ script = "restart.sh", args = ["{{{{webhook.host}}}}"] }}

[[webhooks]]
id = "{SAFE_SYNC}"
title = "Safe sync"
variables = ["host"]
action = "script"
run = {{ script = "restart.sh", args = ["--", "{{{{webhook.host}}}}"] }}

[[webhooks]]
id = "{DEPLOY}"
title = "Deploy"
variables = ["branch"]
action = "script"
run = {{ script = "restart.sh", args = ["{{{{webhook.branch}}}}"] }}
"#
    )
}

pub fn ready() -> Api {
    let api = api(&file());
    write_script(
        &api.folder.path().join("scripts"),
        "input.sh",
        "cat; echo; env | grep PORTAL_WEBHOOK || true",
    );
    start(&api.feature);
    api
}

pub async fn call(api: &Api, hook: &str, content_type: &str, body: &str) -> Option<u64> {
    let request = Request::post(format!("/webhook/{hook}"))
        .header(CONTENT_TYPE, content_type)
        .body(Body::from(body.to_string()))
        .unwrap();
    let (status, _, answer) = send(api, request).await;
    assert_eq!(status, StatusCode::ACCEPTED, "{answer}");
    answer["run_id"].as_str().map(|id| id.parse().unwrap())
}

pub async fn finished(api: &Api, run_id: u64) -> Value {
    let sink = api.feature.state.sink.clone();
    assert!(eventually(|| sink.journal.find(run_id).is_some()).await);
    send(api, get(&format!("/api/automations/runs/{run_id}")))
        .await
        .2
}

pub async fn last_of(api: &Api, automation: &str) -> Value {
    let sink = api.feature.state.sink.clone();
    let run_id = {
        assert!(eventually(|| sink.journal.last_of(automation).is_some()).await);
        sink.journal.last_of(automation).unwrap().id
    };
    finished(api, run_id).await
}
