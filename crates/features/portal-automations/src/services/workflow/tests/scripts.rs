use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::json;
use tokio::sync::watch;

use super::running::{FakeActions, Outcome, run_scripted};
use crate::types::{Ending, StepOutcome};

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
    assert_eq!(
        outcome.trace.entries[0].output.as_deref(),
        Some("args: -- nas\nfrom w\n")
    );
    assert_eq!(outcome.trace.entries[0].detail, "say.sh exited 0");
}

#[tokio::test]
async fn a_failing_script_fails_the_step_with_its_exit_code() {
    let outcome = scripted(
        &workflow(30, "script = \"fail.sh\""),
        &[("fail.sh", "echo 'disk full' >&2; exit 3")],
    )
    .await;
    assert_eq!(outcome.ending, Ending::Failed("fail.sh exited 3".into()));
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
