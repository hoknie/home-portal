use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use portal_config::ConfigStore;
use portal_feature::{EventName, Feature, PortalEvent, StatusChange};
use tempfile::TempDir;
use time::OffsetDateTime;

use super::AutomationsFeature;
use crate::ports::Directory;
use crate::types::{Choice, Outcome, RunRecord, StopAnswer};

pub struct FakeDirectory;

impl Directory for FakeDirectory {
    fn services(&self) -> Vec<Choice> {
        vec![Choice {
            id: "nas".into(),
            name: "NAS".into(),
        }]
    }

    fn users(&self) -> Vec<String> {
        vec!["admin".into()]
    }

    fn environments(&self) -> Vec<String> {
        vec!["home".into(), "internet".into()]
    }
}

pub fn portal(configuration: &str, scripts: &[(&str, &str)]) -> (TempDir, AutomationsFeature) {
    let folder = TempDir::new().unwrap();
    let path = folder.path().join("home-portal.toml");
    fs::write(&path, configuration).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    let root = folder.path().join("scripts");
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o755)).unwrap();
    for (name, body) in scripts {
        write_script(&root, name, body);
    }
    let store = Arc::new(ConfigStore::open(&path).unwrap());
    let feature = AutomationsFeature::new(store, Arc::new(FakeDirectory));
    (folder, feature)
}

pub fn write_script(root: &Path, name: &str, body: &str) {
    let path = root.join(name);
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
}

pub fn start(feature: &AutomationsFeature) {
    for task in feature.loops() {
        tokio::spawn(task);
    }
}

pub async fn eventually(check: impl Fn() -> bool) -> bool {
    let began = Instant::now();
    while began.elapsed() < Duration::from_secs(10) {
        if check() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    false
}

fn automation(id: &str, when: &str, script: &str) -> String {
    format!(
        "[[automations]]\nid = \"{id}\"\ntitle = \"{id}\"\nwhen = {when}\nrun = {{ script = \"{script}\", timeout_seconds = 120 }}\n\n"
    )
}

fn runs(feature: &AutomationsFeature) -> Vec<RunRecord> {
    feature.state.sink.journal.runs(None)
}

#[tokio::test]
async fn at_most_four_runs_execute_at_once() {
    let text: String = (0..6)
        .map(|index| {
            automation(
                &format!("slow{index}"),
                "{ event = \"portal.started\" }",
                "slow.sh",
            )
        })
        .collect();
    let (_folder, feature) = portal(&text, &[("slow.sh", "sleep 1")]);
    start(&feature);
    let sink = feature.state.sink.clone();
    portal_feature::EventSink::emit(
        sink.as_ref(),
        PortalEvent::portal(EventName::PortalStarted, "", OffsetDateTime::now_utc()),
    );
    let mut most = 0;
    let began = Instant::now();
    while runs(&feature).len() < 6 && began.elapsed() < Duration::from_secs(10) {
        most = most.max(sink.groups.count());
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert_eq!(most, 4);
    assert!(
        runs(&feature)
            .iter()
            .all(|run| run.result.outcome == Outcome::Succeeded)
    );
}

#[tokio::test]
async fn settling_kills_a_run_that_hangs_and_returns_in_time() {
    let text = automation("stop", "{ event = \"portal.stopping\" }", "hang.sh");
    let (_folder, feature) = portal(&text, &[("hang.sh", "sleep 60")]);
    start(&feature);
    let events = feature.events();
    events.emit(PortalEvent::portal(
        EventName::PortalStopping,
        "",
        OffsetDateTime::now_utc(),
    ));
    let sink = feature.state.sink.clone();
    assert!(eventually(|| sink.groups.count() == 1).await);
    let began = Instant::now();
    events.settle(Duration::from_secs(1)).await;
    let waited = began.elapsed();
    assert!(waited >= Duration::from_millis(900) && waited < Duration::from_secs(2));
    assert!(eventually(|| sink.groups.count() == 0).await);
    assert!(eventually(|| runs(&feature).len() == 1).await);
}

#[tokio::test]
async fn a_hand_edit_produces_one_configuration_change() {
    let text = automation("audit", "{ event = \"configuration.changed\" }", "echo.sh");
    let (folder, feature) = portal(&text, &[("echo.sh", "echo \"$PORTAL_EVENT_NAME\"")]);
    start(&feature);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let path = folder.path().join("home-portal.toml");
    fs::write(&path, format!("{text}# edited by hand\n")).unwrap();
    assert!(eventually(|| runs(&feature).len() == 1).await);
    tokio::time::sleep(Duration::from_millis(1500)).await;
    let found = runs(&feature);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].event(), "configuration.changed");
    assert_eq!(found[0].result.stdout.text(), "configuration.changed\n");
}

#[tokio::test]
async fn a_status_change_through_the_observer_runs_the_matching_automation() {
    let text = automation(
        "restart-nas",
        "{ event = \"service.status-changed\", services = [\"nas\"], to = [\"down\"] }",
        "restart.sh",
    );
    let (_folder, feature) = portal(&text, &[("restart.sh", "echo \"$1\"; exit 3")]);
    start(&feature);
    feature.observer().changed(&StatusChange {
        service: "nas".into(),
        name: "NAS".into(),
        was: "up".into(),
        now: "down".into(),
        error: Some("refused".into()),
        diagnosis: Some("refused".into()),
        notify: false,
    });
    assert!(eventually(|| runs(&feature).len() == 1).await);
    let run = &runs(&feature)[0];
    assert_eq!(run.result.outcome, Outcome::Failed);
    assert_eq!(run.result.exit_code, Some(3));
    assert_eq!(run.automation, "restart-nas");
}

#[tokio::test]
async fn a_script_outside_the_directory_is_refused_without_a_process() {
    let text = automation("evil", "{ event = \"portal.started\" }", "evil.sh");
    let (folder, feature) = portal(&text, &[]);
    std::os::unix::fs::symlink("/bin/sh", folder.path().join("scripts/evil.sh")).unwrap();
    start(&feature);
    feature.events().emit(PortalEvent::portal(
        EventName::PortalStarted,
        "",
        OffsetDateTime::now_utc(),
    ));
    assert!(eventually(|| runs(&feature).len() == 1).await);
    let run = &runs(&feature)[0];
    assert_eq!(run.result.outcome, Outcome::Refused);
    assert!(run.result.reason.as_deref().unwrap().contains("outside"));
}

#[tokio::test]
async fn the_run_journal_is_kept_across_a_restart_and_numbering_goes_on() {
    let text = automation("hello", "{ event = \"portal.started\" }", "echo.sh");
    let (folder, feature) = portal(&text, &[("echo.sh", "echo hi")]);
    start(&feature);
    feature.events().emit(PortalEvent::portal(
        EventName::PortalStarted,
        "",
        OffsetDateTime::now_utc(),
    ));
    assert!(eventually(|| runs(&feature).len() == 1).await);
    feature.stop();
    let first = runs(&feature)[0].id;
    let store = Arc::new(ConfigStore::open(folder.path().join("home-portal.toml")).unwrap());
    let again = AutomationsFeature::new(store, Arc::new(FakeDirectory));
    let restored = again.state.sink.journal.runs(None);
    assert_eq!(restored.len(), 1);
    assert_eq!(restored[0].id, first);
    assert_eq!(restored[0].result.stdout.text(), "hi\n");
    assert_eq!(
        again
            .state
            .sink
            .run_now(&again.state.sink.cache.find("hello").unwrap(), "admin"),
        first + 1
    );
}

fn stoppable(id: &str, script: &str) -> String {
    format!(
        "[[automations]]\nid = \"{id}\"\ntitle = \"{id}\"\nwhen = {{ event = \"manual\" }}\nrun = {{ script = \"{script}\", timeout_seconds = 900 }}\n\n"
    )
}

fn run_now(feature: &AutomationsFeature, id: &str) -> u64 {
    let sink = &feature.state.sink;
    sink.run_now(&sink.cache.find(id).unwrap(), "admin")
}

#[tokio::test]
async fn a_queued_run_is_active_until_it_finishes() {
    let text = stoppable("slow", "slow.sh");
    let (_folder, feature) = portal(&text, &[("slow.sh", "echo copying; sleep 1")]);
    let id = run_now(&feature, "slow");
    let sink = feature.state.sink.clone();
    assert_eq!(sink.active.find(id).map(|run| run.running()), Some(false));
    start(&feature);
    assert!(eventually(|| sink.active.find(id).is_some_and(|run| run.running())).await);
    assert!(
        eventually(|| sink
            .active
            .find(id)
            .is_some_and(|run| run.control.output().0.text() == "copying\n"))
        .await
    );
    assert!(sink.journal.find(id).is_none());
    assert!(eventually(|| sink.journal.find(id).is_some()).await);
    assert!(sink.active.find(id).is_none());
    assert_eq!(
        sink.journal.find(id).unwrap().result.outcome,
        Outcome::Succeeded
    );
}

#[tokio::test]
async fn stopping_a_script_that_hangs_records_who_stopped_it() {
    let text = stoppable("sync", "sync.sh");
    let (_folder, feature) = portal(&text, &[("sync.sh", "sleep 600 & sleep 600")]);
    start(&feature);
    let id = run_now(&feature, "sync");
    let sink = feature.state.sink.clone();
    assert!(eventually(|| sink.groups.count() == 1).await);
    let began = Instant::now();
    assert_eq!(sink.stop(id, "alice"), StopAnswer::Stopping);
    assert_eq!(sink.stop(id, "alice"), StopAnswer::AlreadyStopping);
    assert!(eventually(|| sink.journal.find(id).is_some()).await);
    assert!(began.elapsed() < Duration::from_secs(1));
    let run = sink.journal.find(id).unwrap();
    assert_eq!(run.result.outcome, Outcome::Stopped);
    assert_eq!(run.result.reason.as_deref(), Some("stopped by alice"));
    assert_eq!(sink.stop(id, "alice"), StopAnswer::Finished);
    assert_eq!(sink.stop(id + 100, "alice"), StopAnswer::Unknown);
    assert!(eventually(|| !sink.gatekeeper.busy()).await);
    let again = run_now(&feature, "sync");
    assert!(sink.active.find(again).is_some());
}

#[tokio::test]
async fn a_stopped_queued_run_never_starts() {
    let mut text: String = (0..4)
        .map(|index| stoppable(&format!("slow{index}"), "slow.sh"))
        .collect();
    text.push_str(&stoppable("backup", "backup.sh"));
    let (folder, feature) = portal(
        &text,
        &[("slow.sh", "sleep 600"), ("backup.sh", "touch started")],
    );
    start(&feature);
    let sink = feature.state.sink.clone();
    for index in 0..4 {
        run_now(&feature, &format!("slow{index}"));
    }
    assert!(eventually(|| sink.groups.count() == 4).await);
    let id = run_now(&feature, "backup");
    assert_eq!(sink.stop(id, "alice"), StopAnswer::Stopping);
    let run = sink.journal.find(id).unwrap();
    assert_eq!(run.result.outcome, Outcome::Stopped);
    assert!(sink.active.find(id).is_none());
    assert!(!sink.gatekeeper.busy_with("backup"));
    sink.groups.kill_all();
    assert!(eventually(|| sink.groups.count() == 0).await);
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(!folder.path().join("scripts/started").exists());
}
