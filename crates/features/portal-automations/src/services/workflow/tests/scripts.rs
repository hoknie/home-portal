use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::json;
use tokio::sync::watch;

use super::running::{FakeActions, Outcome, run_prepared, run_scripted};
use crate::services::workflow::{SecretLookup, Secrets};
use crate::types::{Ending, StepOutcome, Streams};

fn workflow(timeout: u64, step: &str) -> String {
    format!(
        "[[workflows]]\nid = \"w\"\ntitle = \"W\"\ninputs = [\"service\"]\ntimeout_seconds = {timeout}\n[[workflows.steps]]\nid = \"run\"\nkind = \"script\"\n{step}\n"
    )
}

async fn scripted(text: &str, scripts: &[(&str, &str)]) -> Outcome {
    let (_sender, stop) = watch::channel(false);
    run_scripted(text, (Arc::new(FakeActions::default()), stop), scripts).await
}

#[tokio::test]
async fn a_script_step_gets_its_arguments_and_event_fields_and_keeps_its_output() {
    let outcome = scripted(
        &workflow(
            30,
            "script = \"say.sh\"\nargs = [\"--\", \"{{inputs.service}}\"]",
        ),
        &[(
            "say.sh",
            "echo \"args: $*\"; echo \"from $PORTAL_AUTOMATION_ID\"",
        )],
    )
    .await;
    assert_eq!(outcome.ending, Ending::Succeeded(None));
    let result = &outcome.frame.steps["run"];
    assert_eq!(result["exit_code"], json!(0));
    assert_eq!(result["stdout"], json!("args: -- nas\nfrom w\n"));
    let streams = outcome.trace.entries[0].streams.as_ref().unwrap();
    assert_eq!(streams.stdout.text(), "args: -- nas\nfrom w\n");
    assert_eq!(streams.command, vec!["say.sh", "--", "nas"]);
    assert_eq!(outcome.trace.entries[0].output, None);
    assert_eq!(outcome.trace.entries[0].detail, "say.sh exited 0");
}

#[tokio::test]
async fn a_failing_script_fails_the_step_with_its_exit_code() {
    let outcome = scripted(
        &workflow(30, "script = \"fail.sh\""),
        &[("fail.sh", "echo 'disk full' >&2; exit 3")],
    )
    .await;
    assert_eq!(
        outcome.ending,
        Ending::Failed("fail.sh exited 3: disk full".into())
    );
    assert_eq!(outcome.trace.entries[0].outcome, StepOutcome::Failed);
    assert_eq!(outcome.frame.steps["run"]["stderr"], json!("disk full\n"));
}

#[tokio::test]
async fn a_script_that_outlives_the_workflow_is_killed_and_the_run_times_out() {
    let began = Instant::now();
    let outcome = scripted(
        &workflow(1, "script = \"slow.sh\"\ntimeout_seconds = 60"),
        &[("slow.sh", "sleep 30")],
    )
    .await;
    assert_eq!(outcome.ending, Ending::TimedOut);
    assert!(began.elapsed() < Duration::from_secs(10));
}

#[tokio::test]
async fn a_missing_script_fails_with_the_refusal() {
    let outcome = scripted(&workflow(30, "script = \"nowhere.sh\""), &[]).await;
    let Ending::Failed(reason) = &outcome.ending else {
        panic!("{:?}", outcome.ending);
    };
    assert!(reason.contains("nowhere.sh"), "{reason}");
}

#[tokio::test]
async fn a_script_reads_named_variables_and_a_value_on_its_standard_input() {
    let outcome = scripted(
        "[[workflows]]\nid = \"w\"\ntitle = \"W\"\ninputs = [\"service\"]\n[[workflows.steps]]\nid = \"report\"\nkind = \"set\"\nvariable = \"report\"\nobject = { host = \"{{inputs.service}}\" }\n[[workflows.steps]]\nid = \"run\"\nkind = \"script\"\nscript = \"read.sh\"\nenv = { TARGET = \"{{inputs.service}}\" }\nstdin = \"{{vars.report}}\"\n",
        &[("read.sh", "echo \"target=$TARGET\"; cat")],
    )
    .await;
    assert_eq!(outcome.ending, Ending::Succeeded(None));
    assert_eq!(
        outcome.frame.steps["run"]["stdout"],
        json!("target=nas\n{\"host\":\"nas\"}")
    );
}

#[test]
fn reserved_and_malformed_variable_names_are_refused() {
    let found = super::support::fields(&workflow(
        30,
        "script = \"say.sh\"\nenv = { PORTAL_SERVICE_ID = \"x\", PATH = \"/tmp\", lower = \"y\", GOOD_ONE = \"z\" }",
    ));
    assert_eq!(
        found,
        vec![
            "workflows[0].steps[0].env.PATH",
            "workflows[0].steps[0].env.PORTAL_SERVICE_ID",
            "workflows[0].steps[0].env.lower",
        ]
    );
}

#[tokio::test]
async fn a_script_that_fails_at_the_end_of_a_long_output_names_its_last_error_line() {
    let outcome = scripted(
        &workflow(30, "script = \"restart.sh\""),
        &[(
            "restart.sh",
            "head -c 40960 /dev/zero | tr '\\0' 'x'; printf '\\033[31mcontainer not found\\033[0m\\n\\n' >&2; exit 1",
        )],
    )
    .await;
    let entry = &outcome.trace.entries[0];
    assert_eq!(entry.detail, "restart.sh exited 1: container not found");
    let streams = entry.streams.as_ref().unwrap();
    assert_eq!(
        streams.stderr.text(),
        "\u{1b}[31mcontainer not found\u{1b}[0m\n\n"
    );
    assert_eq!(streams.stdout.kept_bytes(), Streams::KEPT_PER_STREAM);
    assert!(streams.stdout.truncated());
    assert_eq!(streams.stdout.total, 40960);
    assert!(!streams.budget_reached);
}

#[tokio::test]
async fn a_script_without_standard_error_is_explained_by_its_standard_output() {
    let outcome = scripted(
        &workflow(30, "script = \"check.sh\""),
        &[("check.sh", "echo 'checking'; echo 'no space left'; exit 2")],
    )
    .await;
    assert_eq!(
        outcome.trace.entries[0].detail,
        "check.sh exited 2: no space left"
    );
}

#[tokio::test]
async fn a_secret_in_the_output_and_the_command_is_masked() {
    let lookup: SecretLookup = Arc::new(|key: &str| (key == "token").then(|| "s3cr3t".to_string()));
    let (_sender, stop) = watch::channel(false);
    let outcome = run_prepared(
        &workflow(30, "script = \"leak.sh\"\nargs = [\"{{secrets.token}}\"]"),
        (Arc::new(FakeActions::default()), stop),
        &[(
            "leak.sh",
            "echo \"token is $1\"; echo \"bad $1\" >&2; exit 1",
        )],
        Secrets::new(lookup),
    )
    .await;
    let entry = &outcome.trace.entries[0];
    let streams = entry.streams.as_ref().unwrap();
    assert_eq!(streams.stdout.text(), "token is ***\n");
    assert_eq!(streams.stderr.text(), "bad ***\n");
    assert_eq!(streams.command, vec!["leak.sh", "***"]);
    assert_eq!(entry.detail, "leak.sh exited 1: bad ***");
}

#[tokio::test]
async fn the_output_budget_of_a_run_keeps_the_later_entries_short() {
    let outcome = scripted(
        "[[workflows]]\nid = \"w\"\ntitle = \"W\"\ninputs = [\"service\"]\n[[workflows.steps]]\nid = \"again\"\nkind = \"loop\"\nrepeat = 20\nbody = [{ id = \"run\", kind = \"script\", script = \"chatty.sh\" }]\n",
        &[("chatty.sh", "head -c 10240 /dev/zero | tr '\\0' 'x'")],
    )
    .await;
    assert_eq!(outcome.ending, Ending::Succeeded(None));
    let kept: Vec<&Streams> = outcome
        .trace
        .entries
        .iter()
        .filter_map(|entry| entry.streams.as_ref())
        .collect();
    assert_eq!(kept.len(), 20);
    let full = kept.iter().filter(|streams| !streams.budget_reached);
    assert!(full.map(|streams| streams.bytes()).sum::<usize>() <= Streams::MOST_BYTES_PER_RUN);
    let last = kept.last().unwrap();
    assert!(last.budget_reached);
    assert_eq!(last.stdout.kept_bytes(), Streams::KEPT_PAST_BUDGET);
    assert_eq!(last.stdout.total, 10240);
    assert!(
        kept.iter().map(|streams| streams.bytes()).sum::<usize>() <= Streams::MOST_BYTES_PER_RUN
    );
}

#[tokio::test]
async fn a_script_error_a_condition_handles_lets_the_run_go_on() {
    let outcome = scripted(
        "[[workflows]]\nid = \"w\"\ntitle = \"W\"\ninputs = [\"service\"]\n[[workflows.steps]]\nid = \"restart\"\nkind = \"script\"\nscript = \"restart.sh\"\nfail_on_error = false\n[[workflows.steps]]\nid = \"check\"\nkind = \"if\"\ncondition = { left = \"{{steps.restart.exit_code}}\", op = \"==\", right = \"0\" }\nthen = [{ id = \"fine\", kind = \"nothing\" }]\nelse = [{ id = \"tell\", kind = \"log\", message = \"restart failed\" }]\n",
        &[("restart.sh", "echo 'container not found' >&2; exit 1")],
    )
    .await;
    assert_eq!(outcome.ending, Ending::Succeeded(None));
    let restart = &outcome.trace.entries[0];
    assert_eq!(restart.outcome, StepOutcome::Succeeded);
    assert_eq!(restart.detail, "restart.sh exited 1: container not found");
    assert!(
        restart
            .log
            .lines
            .iter()
            .any(|line| line.ends_with("exit 1, tolerated")),
        "{:?}",
        restart.log.lines
    );
    assert_eq!(outcome.frame.steps["restart"]["exit_code"], json!(1));
    assert_eq!(outcome.trace.entries[1].detail, "else");
    assert!(
        outcome
            .trace
            .entries
            .iter()
            .any(|entry| entry.step == "tell")
    );
}

#[tokio::test]
async fn a_script_that_fails_the_run_by_default_still_does() {
    let outcome = scripted(
        &workflow(30, "script = \"restart.sh\""),
        &[("restart.sh", "echo 'container not found' >&2; exit 1")],
    )
    .await;
    assert_eq!(
        outcome.ending,
        Ending::Failed("restart.sh exited 1: container not found".into())
    );
}

#[tokio::test]
async fn a_tolerant_script_that_runs_past_its_own_timeout_still_fails() {
    let outcome = scripted(
        &workflow(
            30,
            "script = \"slow.sh\"\ntimeout_seconds = 1\nfail_on_error = false",
        ),
        &[("slow.sh", "sleep 10")],
    )
    .await;
    assert!(
        matches!(&outcome.ending, Ending::Failed(reason) if reason.contains("timed out")),
        "{:?}",
        outcome.ending
    );
    assert_eq!(outcome.trace.entries[0].outcome, StepOutcome::Failed);
}

#[tokio::test]
async fn a_long_error_line_is_shortened_in_the_detail_and_kept_whole_in_the_stream() {
    let long = "x".repeat(400);
    let outcome = scripted(
        &workflow(30, "script = \"loud.sh\""),
        &[("loud.sh", &format!("echo '{long}' >&2; exit 2"))],
    )
    .await;
    let entry = &outcome.trace.entries[0];
    let reason = entry.detail.strip_prefix("loud.sh exited 2: ").unwrap();
    assert_eq!(reason.chars().count(), 120);
    assert!(reason.ends_with('…'));
    assert!(
        entry
            .streams
            .as_ref()
            .unwrap()
            .stderr
            .text()
            .contains(&long)
    );
}
