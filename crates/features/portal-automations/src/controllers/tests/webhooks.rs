use std::fs;

use axum::body::Body;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};

use super::automations::{Api, api, get, revision, send, write};
use crate::features::tests::{eventually, start, write_script};
use crate::helpers::token_hash;

const DEPLOY: &str = "7d3f2a4e-5b1c-4e8f-9a2d-6c0b1e3f4a5d";
const MOTION: &str = "0b9e8c2a-1d3f-4a5b-8c7d-9e0f1a2b3c4d";
const TOKEN: &str = "secret-token";

fn file() -> String {
    format!(
        r#"[[webhooks]]
id = "{DEPLOY}"
title = "Deploy"
variables = ["branch"]
action = "script"
run = {{ script = "deploy.sh", args = ["--", "{{{{webhook.branch}}}}"] }}
token_sha256 = "{hash}"

[[webhooks]]
id = "{MOTION}"
title = "Motion"
action = "event"

[[automations]]
id = "on-motion"
title = "On motion"
when = {{ event = "webhook.received", webhooks = ["{MOTION}"] }}
run = {{ script = "deploy.sh", args = ["{{{{webhook.title}}}}"] }}

[[automations]]
id = "on-deploy"
title = "On deploy"
when = {{ event = "webhook.received", webhooks = ["{DEPLOY}"] }}
run = {{ script = "deploy.sh" }}
"#,
        hash = token_hash(TOKEN)
    )
}

fn ready() -> Api {
    let api = api(&file());
    write_script(
        &api.folder.path().join("scripts"),
        "deploy.sh",
        "printf '%s\\n' \"$@\"",
    );
    api
}

fn call(id: &str, headers: &[(&str, &str)], body: &str) -> Request<Body> {
    let mut builder =
        Request::post(format!("/webhook/{id}")).header(CONTENT_TYPE, "application/json");
    for (name, value) in headers {
        builder = builder.header(*name, *value);
    }
    builder.body(Body::from(body.to_string())).unwrap()
}

fn bearer() -> String {
    format!("Bearer {TOKEN}")
}

#[tokio::test]
async fn a_script_webhook_with_its_token_runs_its_script_with_the_variable() {
    let api = ready();
    start(&api.feature);
    let (status, _, body) = send(
        &api,
        call(
            DEPLOY,
            &[(AUTHORIZATION.as_str(), &bearer())],
            r#"{"branch":"main"}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let run_id = body["run_id"].as_str().unwrap().to_string();
    let sink = api.feature.state.sink.clone();
    assert!(eventually(|| !sink.journal.runs(Some(DEPLOY)).is_empty()).await);
    let run = &sink.journal.runs(Some(DEPLOY))[0];
    assert_eq!(run.id.to_string(), run_id);
    assert_eq!(run.arguments, vec!["--", "main"]);
    assert_eq!(run.result.stdout.text(), "--\nmain\n");
    assert!(sink.journal.runs(Some("on-deploy")).is_empty());
}

#[tokio::test]
async fn a_missing_or_wrong_token_is_refused_and_nothing_runs() {
    let api = ready();
    for headers in [
        vec![],
        vec![(AUTHORIZATION.as_str(), "Bearer other")],
        vec![("x-webhook-token", "other")],
    ] {
        let (status, _, body) = send(&api, call(DEPLOY, &headers, r#"{"branch":"main"}"#)).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert!(!body.to_string().contains(TOKEN));
    }
    let (status, _, _) = send(
        &api,
        call(
            DEPLOY,
            &[("x-webhook-token", TOKEN)],
            r#"{"branch":"main"}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(api.feature.state.webhooks.last(DEPLOY).unwrap().status, 202);
}

#[tokio::test]
async fn a_missing_variable_is_named_and_a_query_variable_counts() {
    let api = ready();
    let (status, _, body) = send(
        &api,
        call(DEPLOY, &[(AUTHORIZATION.as_str(), &bearer())], "{}"),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["errors"][0]["field"], "branch");
    let request = Request::post(format!("/webhook/{DEPLOY}?branch=dev"))
        .header(AUTHORIZATION, bearer())
        .body(Body::empty())
        .unwrap();
    let (status, _, _) = send(&api, request).await;
    assert_eq!(status, StatusCode::ACCEPTED);
}

#[tokio::test]
async fn an_unknown_or_disabled_webhook_is_not_found_and_a_large_body_is_refused() {
    let api = ready();
    let (status, _, _) = send(
        &api,
        call("00000000-0000-4000-8000-000000000000", &[], "{}"),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let large = format!("{{\"branch\":\"{}\"}}", "x".repeat(70 * 1024));
    let (status, _, _) = send(
        &api,
        call(DEPLOY, &[(AUTHORIZATION.as_str(), &bearer())], &large),
    )
    .await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn the_sixty_first_call_in_a_minute_is_throttled() {
    let api = ready();
    for _ in 0..60 {
        let (status, _, _) = send(&api, call(MOTION, &[], "")).await;
        assert_eq!(status, StatusCode::ACCEPTED);
    }
    let (status, _, _) = send(&api, call(MOTION, &[], "")).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn an_event_webhook_starts_only_the_automations_that_filter_it() {
    let api = ready();
    start(&api.feature);
    let (status, _, body) = send(&api, call(MOTION, &[], "")).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(body, json!({ "accepted": true }));
    let sink = api.feature.state.sink.clone();
    assert!(eventually(|| !sink.journal.runs(Some("on-motion")).is_empty()).await);
    assert_eq!(
        sink.journal.runs(Some("on-motion"))[0].arguments,
        vec!["Motion"]
    );
    assert!(sink.journal.runs(Some("on-deploy")).is_empty());
}

#[tokio::test]
async fn a_token_is_shown_once_and_only_its_hash_is_kept() {
    let api = ready();
    let current = revision(&api).await;
    let (status, _, created) = send(
        &api,
        write(
            "POST",
            "/api/webhooks",
            Some(&current),
            r#"{"title":"Phone","variables":["name"],"action":"event","with_token":true}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let token = created["token"].as_str().unwrap().to_string();
    let id = created["webhook"]["id"].as_str().unwrap().to_string();
    assert_eq!(token.len(), 64);
    assert_eq!(created["webhook"]["address"], format!("/webhook/{id}"));
    let text = fs::read_to_string(api.folder.path().join("home-portal.toml")).unwrap();
    assert!(text.contains(&token_hash(&token)));
    assert!(!text.contains(&token));
    let (_, _, listed) = send(&api, get("/api/webhooks")).await;
    assert!(!listed.to_string().contains(&token));
    let phone: &Value = listed["webhooks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|webhook| webhook["id"] == id)
        .unwrap();
    assert_eq!(phone["protected"], true);
    let current = revision(&api).await;
    let (status, _, _) = send(
        &api,
        write(
            "DELETE",
            &format!("/api/webhooks/{id}/token"),
            Some(&current),
            "",
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _, _) = send(&api, call(&id, &[], r#"{"name":"x"}"#)).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let current = revision(&api).await;
    let (_, _, reissued) = send(
        &api,
        write(
            "POST",
            &format!("/api/webhooks/{id}/token"),
            Some(&current),
            "",
        ),
    )
    .await;
    let fresh = reissued["token"].as_str().unwrap();
    assert_ne!(fresh, token);
    let (status, _, _) = send(&api, call(&id, &[], r#"{"name":"x"}"#)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_webhook_is_edited_and_deleted_with_the_revision() {
    let api = ready();
    let (status, _, _) = send(
        &api,
        write("PUT", &format!("/api/webhooks/{MOTION}"), None, "{}"),
    )
    .await;
    assert_eq!(status, StatusCode::PRECONDITION_REQUIRED);
    let current = revision(&api).await;
    let (status, _, body) = send(
        &api,
        write(
            "PUT",
            &format!("/api/webhooks/{MOTION}"),
            Some(&current),
            r#"{"title":"Motion at the door","action":"event","variables":["camera"]}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["variables"], json!(["camera"]));
    let current = revision(&api).await;
    let (status, _, body) = send(
        &api,
        write(
            "DELETE",
            &format!("/api/webhooks/{DEPLOY}"),
            Some(&current),
            "",
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["webhooks"].as_array().unwrap().len(), 1);
    let (status, _, _) = send(
        &api,
        call(
            DEPLOY,
            &[(AUTHORIZATION.as_str(), &bearer())],
            r#"{"branch":"main"}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_refused_call_does_not_hide_the_last_accepted_one() {
    let api = ready();
    let (status, _, _) = send(
        &api,
        call(
            DEPLOY,
            &[(AUTHORIZATION.as_str(), &bearer())],
            r#"{"branch":"main"}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    send(&api, call(DEPLOY, &[], r#"{"branch":"main"}"#)).await;
    assert_eq!(api.feature.state.webhooks.last(DEPLOY).unwrap().status, 202);
}

#[tokio::test]
async fn a_call_while_the_portal_stops_is_refused_as_unavailable() {
    let api = ready();
    portal_feature::EventSink::emit(
        api.feature.state.sink.as_ref(),
        portal_feature::PortalEvent::portal(
            portal_feature::EventName::PortalStopping,
            "",
            time::OffsetDateTime::UNIX_EPOCH,
        ),
    );
    let (status, _, _) = send(&api, call(MOTION, &[], "")).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn the_client_address_comes_from_the_portal_not_the_proxy() {
    let api = ready();
    start(&api.feature);
    let mut request = call(MOTION, &[], "");
    request
        .extensions_mut()
        .insert(portal_feature::ClientAddress(
            "203.0.113.9".parse().unwrap(),
        ));
    send(&api, request).await;
    let sink = api.feature.state.sink.clone();
    assert!(eventually(|| !sink.journal.runs(Some("on-motion")).is_empty()).await);
    let run = &sink.journal.runs(Some("on-motion"))[0];
    assert!(
        run.fields
            .contains(&("client.address".to_string(), "203.0.113.9".to_string()))
    );
}

#[tokio::test]
async fn tags_are_saved_listed_and_shared_in_the_catalogue() {
    let api = ready();
    let current = revision(&api).await;
    let (status, _, _) = send(
        &api,
        write(
            "PUT",
            &format!("/api/webhooks/{MOTION}"),
            Some(&current),
            r#"{"title":"Motion","action":"event","tags":["camera","door"]}"#,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let current = revision(&api).await;
    let (status, _, _) = send(
        &api,
        write(
            "PUT",
            "/api/automations/on-motion",
            Some(&current),
            &format!(r#"{{"id":"on-motion","title":"On motion","tags":["Door","night"],"when":{{"event":"webhook.received","webhooks":["{MOTION}"]}},"run":{{"script":"deploy.sh","args":["{{{{webhook.title}}}}"]}}}}"#),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let text = fs::read_to_string(api.folder.path().join("home-portal.toml")).unwrap();
    assert!(text.contains("tags = [\"camera\", \"door\"]"), "{text}");
    let (_, _, listed) = send(&api, get("/api/webhooks")).await;
    let motion = listed["webhooks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|webhook| webhook["id"] == MOTION)
        .unwrap()
        .clone();
    assert_eq!(motion["tags"], json!(["camera", "door"]));
    let (_, _, catalogue) = send(&api, get("/api/automations/catalogue")).await;
    assert_eq!(
        catalogue["choices"]["tags"],
        json!(["camera", "Door", "night"])
    );
}
