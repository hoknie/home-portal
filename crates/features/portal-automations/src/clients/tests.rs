use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tempfile::TempDir;

use super::{GroupRegistry, Runner};
use crate::types::{Invocation, Outcome};

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
    let finished = Runner::run(invocation, groups.clone()).await;
    (finished, groups)
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
