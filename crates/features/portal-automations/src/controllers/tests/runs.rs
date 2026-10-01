use axum::http::StatusCode;

use super::automations::{api, get, send, write};
use crate::features::AutomationsFeature;
use crate::features::tests::{eventually, start, write_script};

const FILE: &str = r#"[[automations]]
id = "backup"
title = "Backup"
when = { event = "manual" }
run = { script = "backup.sh", args = ["--target", "nas"], timeout_seconds = 900 }
"#;

fn with_backup(body: &str) -> super::automations::Api {
    let api = api(FILE);
    write_script(&api.folder.path().join("scripts"), "backup.sh", body);
    start(&api.feature);
    api
}

async fn run_backup(api: &super::automations::Api) -> String {
    let (status, _, body) = send(api, write("POST", "/api/automations/backup/run", None, "")).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    body["run_id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn a_run_in_progress_is_listed_first_with_its_output_so_far() {
    let api = with_backup("echo copying; sleep 600");
    let id = run_backup(&api).await;
    let sink = api.feature.state.sink.clone();
    let run_id: u64 = id.parse().unwrap();
    assert!(
        eventually(|| sink
            .active
            .find(run_id)
            .is_some_and(|run| run.control.output().0.text() == "copying\n"))
        .await
    );
    let (status, _, body) = send(&api, get(AutomationsFeature::RUNS)).await;
    assert_eq!(status, StatusCode::OK);
    let run = &body["runs"][0];
    assert_eq!(run["id"], id);
    assert_eq!(run["outcome"]["result"], "running");
    assert_eq!(run["outcome"]["stdout"]["tail"], "");
    assert_eq!(run["arguments"], serde_json::json!(["--target", "nas"]));
    let (_, _, one) = send(&api, get(&format!("/api/automations/runs/{id}"))).await;
    assert_eq!(one["outcome"]["result"], "running");
    assert_eq!(one["outcome"]["stdout"]["tail"], "copying\n");
    let (_, _, listed) = send(&api, get(AutomationsFeature::COLLECTION)).await;
    assert_eq!(listed["automations"][0]["active_run"]["id"], id);
    assert_eq!(
        listed["automations"][0]["last_run"],
        serde_json::Value::Null
    );
    let (_, _, filtered) = send(&api, get("/api/automations/runs?text=NAS")).await;
    assert_eq!(filtered["runs"][0]["id"], id);
    let (_, _, other) = send(&api, get("/api/automations/runs?automation=nope")).await;
    assert_eq!(other["runs"], serde_json::json!([]));
    sink.groups.kill_all();
}

#[tokio::test]
async fn stopping_a_run_answers_202_then_records_it_as_stopped_by_the_person() {
    let api = with_backup("sleep 600");
    let id = run_backup(&api).await;
    let sink = api.feature.state.sink.clone();
    assert!(eventually(|| sink.groups.count() == 1).await);
    let stop = format!("/api/automations/runs/{id}/stop");
    let (status, _, body) = send(&api, write("POST", &stop, None, "")).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(body["id"], id);
    let run_id: u64 = id.parse().unwrap();
    assert!(eventually(|| sink.journal.find(run_id).is_some()).await);
    let (_, _, one) = send(&api, get(&format!("/api/automations/runs/{id}"))).await;
    assert_eq!(one["outcome"]["result"], "stopped");
    assert_eq!(one["outcome"]["reason"], "stopped by admin");
    let (status, _, _) = send(&api, write("POST", &stop, None, "")).await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn an_unknown_run_answers_404() {
    let api = with_backup("true");
    let (status, _, _) = send(&api, get("/api/automations/runs/999")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _, _) = send(&api, get("/api/automations/runs/not-a-number")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _, _) = send(
        &api,
        write("POST", "/api/automations/runs/999/stop", None, ""),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn the_journal_answers_in_pages_with_running_runs_only_on_the_first() {
    let api = with_backup("sleep 600");
    let running = run_backup(&api).await;
    for run in 1..=120u64 {
        api.feature
            .state
            .sink
            .journal
            .record(crate::types::RunRecord::skipped(
                &crate::services::tests::support::pending(&format!("a{run}"), 1000 + run, None),
                crate::types::SkipReason::Cooldown,
                time::OffsetDateTime::UNIX_EPOCH + time::Duration::seconds(run as i64),
            ));
    }
    let (status, _, first) = send(&api, get("/api/automations/runs?limit=50")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(first["runs"].as_array().unwrap().len(), 51);
    assert_eq!(first["runs"][0]["id"], running);
    assert_eq!(first["runs"][1]["id"], "1120");
    assert_eq!(first["next_before"], "1071");
    let (_, _, second) = send(&api, get("/api/automations/runs?limit=50&before=1071")).await;
    assert_eq!(second["runs"].as_array().unwrap().len(), 50);
    assert_eq!(second["runs"][0]["id"], "1070");
    assert!(
        second["runs"]
            .as_array()
            .unwrap()
            .iter()
            .all(|run| run["id"] != running)
    );
    let (_, _, third) = send(
        &api,
        get(&format!(
            "/api/automations/runs?limit=50&before={}",
            second["next_before"].as_str().unwrap()
        )),
    )
    .await;
    assert_eq!(third["runs"].as_array().unwrap().len(), 20);
    assert_eq!(third["next_before"], serde_json::Value::Null);
}

#[tokio::test]
async fn a_page_size_or_cursor_out_of_range_answers_400_naming_it() {
    let api = with_backup("true");
    for (query, field) in [
        ("limit=500", "limit"),
        ("limit=0", "limit"),
        ("before=last", "before"),
    ] {
        let (status, _, body) = send(&api, get(&format!("/api/automations/runs?{query}"))).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{query}");
        assert!(body.to_string().contains(field), "{query}: {body}");
    }
}
