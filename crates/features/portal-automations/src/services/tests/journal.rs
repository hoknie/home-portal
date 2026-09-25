use time::{Duration, OffsetDateTime};

use super::support::pending;
use crate::services::Journal;
use crate::types::{RunRecord, SkipReason};

fn at(seconds: i64) -> OffsetDateTime {
    OffsetDateTime::UNIX_EPOCH + Duration::seconds(seconds)
}

#[test]
fn the_two_hundred_and_first_run_evicts_the_oldest() {
    let journal = Journal::default();
    for run in 0..=200 {
        journal.record(RunRecord::skipped(
            &pending(&format!("a{run}"), run, None),
            SkipReason::Cooldown,
            at(run as i64),
        ));
    }
    let runs = journal.runs(None);
    assert_eq!(runs.len(), 200);
    assert_eq!(runs[0].id, 200);
    assert_eq!(runs[199].id, 1);
}

#[test]
fn consecutive_skips_of_one_automation_merge_with_a_count() {
    let journal = Journal::default();
    for run in 0..50 {
        journal.record(RunRecord::skipped(
            &pending("audit", run, None),
            SkipReason::Cooldown,
            at(run as i64),
        ));
        journal.record(RunRecord::skipped(
            &pending("other", 1000 + run, None),
            SkipReason::RateLimit,
            at(run as i64),
        ));
    }
    let audit = journal.runs(Some("audit"));
    assert_eq!(audit.len(), 1);
    assert_eq!(audit[0].seen.count, 50);
    assert_eq!(audit[0].seen.last_at, at(49));
    assert_eq!(journal.runs(None).len(), 2);
}

#[test]
fn a_manual_run_that_is_dropped_keeps_its_own_record() {
    let journal = Journal::default();
    journal.record(RunRecord::skipped(
        &pending("a", 1, None),
        SkipReason::Dropped,
        at(0),
    ));
    journal.record(RunRecord::skipped(
        &pending("a", 2, Some("admin")),
        SkipReason::Dropped,
        at(1),
    ));
    let runs = journal.runs(Some("a"));
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].id, 2);
    assert_eq!(journal.last_of("a").unwrap().id, 2);
    assert!(journal.runs(Some("b")).is_empty());
}

#[test]
fn the_journal_narrows_by_webhook_and_by_a_text_in_a_field_or_an_argument() {
    use crate::types::RunFilter;

    let journal = Journal::default();
    let mut from_webhook =
        RunRecord::skipped(&pending("on-motion", 1, None), SkipReason::Cooldown, at(0));
    from_webhook
        .fields
        .push(("webhook.id".into(), "hook-1".into()));
    from_webhook
        .fields
        .push(("webhook.branch".into(), "Main".into()));
    journal.record(from_webhook);
    let mut own = RunRecord::skipped(&pending("hook-1", 2, None), SkipReason::Running, at(1));
    own.arguments = vec!["--".into(), "release-7".into()];
    journal.record(own);
    journal.record(RunRecord::skipped(
        &pending("other", 3, None),
        SkipReason::Running,
        at(2),
    ));
    let ids = |filter: RunFilter| {
        journal
            .matching(&filter)
            .iter()
            .map(|run| run.id)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        ids(RunFilter {
            webhook: Some("hook-1".into()),
            ..RunFilter::default()
        }),
        vec![2, 1]
    );
    assert_eq!(
        ids(RunFilter {
            text: Some("main".into()),
            ..RunFilter::default()
        }),
        vec![1]
    );
    assert_eq!(
        ids(RunFilter {
            text: Some("RELEASE".into()),
            ..RunFilter::default()
        }),
        vec![2]
    );
    assert_eq!(
        ids(RunFilter {
            text: Some("  ".into()),
            ..RunFilter::default()
        })
        .len(),
        3
    );
}

#[test]
fn skips_caused_by_different_webhooks_are_kept_apart() {
    let journal = Journal::default();
    for (run, hook) in [(1, "a"), (2, "b")] {
        let mut skipped = RunRecord::skipped(
            &pending("x", run, None),
            SkipReason::Cooldown,
            at(run as i64),
        );
        skipped.fields.push(("webhook.id".into(), hook.into()));
        journal.record(skipped);
    }
    assert_eq!(journal.runs(Some("x")).len(), 2);
}
