use std::sync::Arc;
use std::time::Duration;

use tokio::sync::watch;

use super::super::http::serve;
use super::super::running::{FakeActions, run_peeking};
use crate::services::workflow::Secrets;
use crate::types::{StepLog, StepOutcome, Trace, TraceEntry};

async fn peeked(text: &str, after: Duration) -> (Trace, Trace) {
    let (_sender, stop) = watch::channel(false);
    let (outcome, seen) = run_peeking(
        text,
        (Arc::new(FakeActions::default()), stop),
        &[],
        (Secrets::none(), Some(after)),
    )
    .await;
    (seen.unwrap(), outcome.trace)
}

fn values(entry: &TraceEntry) -> Vec<(&str, &str)> {
    entry
        .log
        .values
        .iter()
        .map(|value| (value.template.as_str(), value.value.as_str()))
        .collect()
}

#[tokio::test]
async fn a_wait_in_progress_shows_its_seconds_and_values_before_it_ends() {
    let (during, after) = peeked(
        "[[workflows]]\nid = \"w\"\ntitle = \"W\"\ninputs = [\"service\"]\n[[workflows.steps]]\nid = \"pause\"\nkind = \"set\"\nvariable = \"pause\"\nvalue = \"2\"\n[[workflows.steps]]\nid = \"nap\"\nkind = \"wait\"\nseconds = \"{{vars.pause}}\"\n",
        Duration::from_millis(600),
    )
    .await;
    let nap = &during.entries[1];
    assert_eq!(nap.outcome, StepOutcome::Running);
    assert_eq!(nap.wait_seconds, Some(2));
    assert_eq!(
        values(nap).first().map(|(template, _)| *template),
        Some("{{vars.pause}}")
    );
    let done = &after.entries[1];
    assert_eq!(done.outcome, StepOutcome::Succeeded);
    assert_eq!(done.wait_seconds, Some(2));
    assert_eq!(values(done), values(nap));
}

#[tokio::test]
async fn a_request_in_progress_already_shows_the_address_it_rendered() {
    let port = serve().await;
    let (during, after) = peeked(
        &format!("[[workflows]]\nid = \"w\"\ntitle = \"W\"\ninputs = [\"service\"]\n[[workflows.steps]]\nid = \"call\"\nkind = \"http\"\nurl = \"http://127.0.0.1:{port}/{{{{inputs.service}}}}/../slow\"\n"),
        Duration::from_millis(1000),
    )
    .await;
    let call = &during.entries[0];
    assert_eq!(call.outcome, StepOutcome::Running);
    assert_eq!(call.wait_seconds, None);
    assert_eq!(
        values(call),
        vec![(
            format!("http://127.0.0.1:{port}/{{{{inputs.service}}}}/../slow").as_str(),
            format!("\"http://127.0.0.1:{port}/nas/../slow\"").as_str()
        )]
    );
    assert!(!after.entries[0].log.lines.is_empty());
}

#[test]
fn a_published_snapshot_touches_only_a_running_entry_and_never_the_log_budget() {
    let mut trace = Trace::default();
    let entry = TraceEntry {
        path: "steps[0]".into(),
        step: "nap".into(),
        label: "nap".into(),
        kind: "wait".into(),
        iteration: None,
        outcome: StepOutcome::Running,
        started_at: time::OffsetDateTime::UNIX_EPOCH,
        duration: Duration::ZERO,
        detail: String::new(),
        output: None,
        shape: None,
        streams: None,
        log: StepLog::default(),
        item: None,
        level: None,
        wait_seconds: None,
    };
    let index = trace.start(entry.clone()).unwrap();
    let mut log = StepLog::default();
    log.push_value("{{vars.pause}}", "2");
    trace.progress(index, Some(log.clone()), Some(2));
    assert_eq!(trace.entries[0].log.values.len(), 1);
    assert_eq!(trace.entries[0].wait_seconds, Some(2));
    assert_eq!(trace.log_bytes, 0);
    let started = time::OffsetDateTime::UNIX_EPOCH;
    let now = started + time::Duration::seconds(12);
    let running = crate::responses::TraceEntryResponse::of(&trace.entries[0], Some(now));
    assert_eq!(running.duration_milliseconds, 12_000);
    assert_eq!(running.wait_seconds, Some(2));
    trace.entries[0].outcome = StepOutcome::Succeeded;
    trace.entries[0].duration = Duration::from_millis(2_004);
    assert_eq!(
        crate::responses::TraceEntryResponse::of(&trace.entries[0], Some(now))
            .duration_milliseconds,
        2_004
    );
    assert_eq!(
        crate::responses::TraceEntryResponse::of(&trace.entries[0], None).duration_milliseconds,
        2_004
    );
    trace.progress(index, Some(StepLog::default()), Some(9));
    assert_eq!(trace.entries[0].wait_seconds, Some(2));
    assert_eq!(trace.entries[0].log.values.len(), 1);
}
