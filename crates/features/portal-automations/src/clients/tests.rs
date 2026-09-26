use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tempfile::TempDir;

use super::{GroupRegistry, Runner};
use crate::types::{Invocation, Outcome, RunControl};

#[derive(Default)]
struct Groups {
    alive: Mutex<Vec<u32>>,
}

impl GroupRegistry for Groups {
    fn started(&self, group: u32) {
        self.alive.lock().unwrap().push(group);
    }

    fn reaped(&self, group: u32) {
        self.alive.lock().unwrap().retain(|alive| *alive != group);
    }
}

fn script(folder: &Path, body: &str) -> PathBuf {
    let path = folder.join("script.sh");
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    path
}

fn invocation(folder: &TempDir, body: &str, arguments: &[&str], timeout: u64) -> Invocation {
    Invocation {
        program: script(folder.path(), body),
        directory: folder.path().to_path_buf(),
        arguments: arguments.iter().map(|text| text.to_string()).collect(),
        environment: vec![("PATH".into(), "/usr/bin:/bin".into())],
        input: "{\"service.id\":\"nas\"}".into(),
        timeout: Duration::from_secs(timeout),
    }
}

async fn run(invocation: &Invocation) -> (crate::types::Finished, Arc<Groups>) {
    let groups = Arc::new(Groups::default());
    let finished = Runner::run(invocation, groups.clone(), RunControl::new().1).await;
    (finished, groups)
}

fn waiting_for(mut check: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !check() {
        assert!(Instant::now() < deadline, "the condition never came true");
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn alive(pid: &str) -> bool {
    std::process::Command::new("/bin/kill")
        .args(["-0", pid.trim()])
        .status()
        .is_ok_and(|status| status.success())
}

#[tokio::test]
async fn an_argument_with_a_space_and_a_quote_arrives_as_one_argument() {
    let folder = TempDir::new().unwrap();
    let (finished, _) = run(&invocation(
        &folder,
        "printf '%s\\n' \"$#\" \"$1\" \"$2\"",
        &["--reason", "connection refused; rm -rf \"/\""],
        10,
    ))
    .await;
    assert_eq!(finished.outcome, Outcome::Succeeded);
    assert_eq!(
        finished.stdout.text(),
        "2\n--reason\nconnection refused; rm -rf \"/\"\n"
    );
}

#[tokio::test]
async fn the_environment_is_clean_and_the_input_is_the_event() {
    let folder = TempDir::new().unwrap();
    let (finished, _) = run(&invocation(&folder, "env | sort; cat", &[], 10)).await;
    let output = finished.stdout.text();
    assert!(output.contains("PATH=/usr/bin:/bin"), "{output}");
    assert!(!output.contains("CARGO"), "{output}");
    assert!(output.ends_with("{\"service.id\":\"nas\"}"), "{output}");
}

#[tokio::test]
async fn a_long_output_keeps_its_end_and_counts_every_byte() {
    let folder = TempDir::new().unwrap();
    let (finished, _) = run(&invocation(
        &folder,
        "head -c 1048576 /dev/zero | tr '\\0' 'x'; echo; echo last-line",
        &[],
        20,
    ))
    .await;
    assert_eq!(finished.outcome, Outcome::Succeeded);
    assert_eq!(finished.stdout.total, 1_048_576 + 1 + 10);
    assert!(finished.stdout.text().ends_with("\nlast-line\n"));
    assert!(finished.stdout.truncated());
}

#[tokio::test]
async fn a_script_still_open_for_writing_starts_once_the_writer_closes_it() {
    let folder = TempDir::new().unwrap();
    let invocation = invocation(&folder, "echo started", &[], 10);
    let writer = fs::OpenOptions::new()
        .append(true)
        .open(&invocation.program)
        .unwrap();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(200)).await;
        drop(writer);
    });
    let (finished, _) = run(&invocation).await;
    assert_eq!(
        finished.outcome,
        Outcome::Succeeded,
        "{:?}",
        finished.reason
    );
    assert_eq!(finished.stdout.text(), "started\n");
}

#[tokio::test]
async fn a_script_past_its_timeout_is_killed_with_its_children() {
    let folder = TempDir::new().unwrap();
    let began = Instant::now();
    let (finished, groups) =
        run(&invocation(&folder, "sleep 60 & echo $!; sleep 60", &[], 1)).await;
    assert_eq!(finished.outcome, Outcome::TimedOut);
    assert!(began.elapsed() < Duration::from_secs(5));
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(!alive(&finished.stdout.text()));
    assert!(groups.alive.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_script_that_exits_ends_its_run_and_its_background_children() {
    let folder = TempDir::new().unwrap();
    let began = Instant::now();
    let (finished, _) = run(&invocation(&folder, "sleep 60 & echo $!; exit 0", &[], 30)).await;
    assert_eq!(finished.outcome, Outcome::Succeeded);
    assert!(began.elapsed() < Duration::from_secs(2));
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(!alive(&finished.stdout.text()));
}

#[tokio::test]
async fn an_exit_code_other_than_zero_fails_with_that_code_and_its_error_output() {
    let folder = TempDir::new().unwrap();
    let (finished, _) = run(&invocation(
        &folder,
        "echo 'disk full' >&2; exit 3",
        &[],
        10,
    ))
    .await;
    assert_eq!(finished.outcome, Outcome::Failed);
    assert_eq!(finished.exit_code, Some(3));
    assert_eq!(finished.stderr.text(), "disk full\n");
}

#[tokio::test]
async fn arguments_beyond_the_system_limit_fail_to_start_with_the_reason() {
    let folder = TempDir::new().unwrap();
    let long = "x".repeat(4 * 1024 * 1024);
    let (finished, _) = run(&invocation(&folder, "exit 0", &[long.as_str()], 10)).await;
    assert_eq!(finished.outcome, Outcome::Failed);
    assert!(
        finished
            .reason
            .as_deref()
            .is_some_and(|reason| reason.starts_with("the script could not start")),
        "{:?}",
        finished.reason
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn output_is_readable_while_the_script_runs() {
    let folder = TempDir::new().unwrap();
    let invocation = invocation(&folder, "echo 'step 1'; sleep 30", &[], 60);
    let (stop, control) = RunControl::new();
    let watched = control.clone();
    let task = tokio::spawn(async move {
        Runner::run(&invocation, Arc::new(Groups::default()), control).await
    });
    tokio::task::spawn_blocking(move || waiting_for(|| watched.output().0.text() == "step 1\n"))
        .await
        .unwrap();
    stop.send_replace(true);
    assert_eq!(task.await.unwrap().outcome, Outcome::Stopped);
}

#[tokio::test]
async fn a_stopped_script_ends_with_its_children_within_a_second() {
    let folder = TempDir::new().unwrap();
    let invocation = invocation(&folder, "sleep 600 & echo $!; sleep 600", &[], 900);
    let (stop, control) = RunControl::new();
    let tails = control.clone();
    let groups = Arc::new(Groups::default());
    let task = tokio::spawn({
        let groups = groups.clone();
        async move { Runner::run(&invocation, groups, control).await }
    });
    while tails.output().0.text().is_empty() {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let began = Instant::now();
    stop.send_replace(true);
    let finished = task.await.unwrap();
    assert!(began.elapsed() < Duration::from_secs(1));
    assert_eq!(finished.outcome, Outcome::Stopped);
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(!alive(&finished.stdout.text()));
    assert!(groups.alive.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_script_that_ignores_sigterm_is_killed_after_the_grace() {
    let folder = TempDir::new().unwrap();
    let invocation = invocation(
        &folder,
        "trap '' TERM; echo ready; while :; do sleep 1; done",
        &[],
        900,
    );
    let (stop, mut control) = RunControl::new();
    control.grace = Duration::from_millis(300);
    let tails = control.clone();
    let task = tokio::spawn(async move {
        Runner::run(&invocation, Arc::new(Groups::default()), control).await
    });
    while tails.output().0.text().is_empty() {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let began = Instant::now();
    stop.send_replace(true);
    let finished = task.await.unwrap();
    assert_eq!(finished.outcome, Outcome::Stopped);
    assert!(began.elapsed() >= Duration::from_millis(300));
    assert!(began.elapsed() < Duration::from_secs(3));
}

#[tokio::test]
async fn a_stop_before_the_spawn_never_starts_the_script() {
    let folder = TempDir::new().unwrap();
    let marker = folder.path().join("started");
    let body = format!("touch '{}'", marker.display());
    let invocation = invocation(&folder, &body, &[], 10);
    let (stop, control) = RunControl::new();
    stop.send_replace(true);
    let finished = Runner::run(&invocation, Arc::new(Groups::default()), control).await;
    assert_eq!(finished.outcome, Outcome::Stopped);
    assert!(!marker.exists());
}

#[tokio::test]
async fn a_stopped_script_that_exits_zero_is_still_stopped() {
    let folder = TempDir::new().unwrap();
    let invocation = invocation(
        &folder,
        "trap 'exit 0' TERM; echo ready; while :; do sleep 0.1; done",
        &[],
        900,
    );
    let (stop, control) = RunControl::new();
    let tails = control.clone();
    let task = tokio::spawn(async move {
        Runner::run(&invocation, Arc::new(Groups::default()), control).await
    });
    while tails.output().0.text().is_empty() {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    stop.send_replace(true);
    let finished = task.await.unwrap();
    assert_eq!(finished.outcome, Outcome::Stopped);
    assert_eq!(finished.exit_code, Some(0));
}
