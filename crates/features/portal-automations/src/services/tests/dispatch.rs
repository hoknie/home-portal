use std::time::{Duration, Instant};

use portal_feature::{EventName, EventSink, PortalEvent, Visitor};
use time::OffsetDateTime;

use super::support::{automation, entry, pending, sink, status_change};
use crate::services::dispatch::{ActiveRuns, Gatekeeper, RunQueue};
use crate::types::{RunFilter, SkipReason};

#[test]
fn a_full_queue_drops_the_oldest_and_counts_it() {
    let queue = RunQueue::default();
    let mut dropped = Vec::new();
    for run in 0..150 {
        if let Some(old) = queue.push(pending(&format!("a{run}"), run, None)) {
            dropped.push(old.run_id);
        }
    }
    assert_eq!(queue.len(), 100);
    assert_eq!(dropped, (0..50).collect::<Vec<_>>());
}

#[test]
fn a_flapping_service_runs_once_inside_its_cooldown() {
    let gatekeeper = Gatekeeper::default();
    let restart = automation(
        "restart",
        "{ event = \"portal.started\" }",
        "cooldown_seconds = 300",
    );
    let began = Instant::now();
    let mut skipped = Vec::new();
    for second in 0..10 {
        let now = began + Duration::from_secs(second * 6);
        match gatekeeper.admit(&restart, now) {
            Ok(()) => {
                gatekeeper.started("restart", now);
                gatekeeper.finished("restart");
            }
            Err(reason) => skipped.push(reason),
        }
    }
    assert_eq!(skipped, vec![SkipReason::Cooldown; 9]);
}

#[test]
fn the_sixty_first_start_in_an_hour_is_limited() {
    let gatekeeper = Gatekeeper::default();
    let loop_prone = automation("a", "{ event = \"configuration.changed\" }", "");
    let began = Instant::now();
    for minute in 0..60 {
        let now = began + Duration::from_secs(minute * 30);
        gatekeeper.admit(&loop_prone, now).unwrap();
        gatekeeper.started("a", now);
        gatekeeper.finished("a");
    }
    let later = began + Duration::from_secs(1800 + 10);
    assert_eq!(
        gatekeeper.admit(&loop_prone, later),
        Err(SkipReason::RateLimit)
    );
    let an_hour_on = began + Duration::from_secs(3600 + 1);
    assert_eq!(gatekeeper.admit(&loop_prone, an_hour_on), Ok(()));
}

#[test]
fn a_running_or_waiting_automation_is_skipped() {
    let gatekeeper = Gatekeeper::default();
    let one = automation("a", "{ event = \"portal.started\" }", "");
    let now = Instant::now();
    gatekeeper.admit(&one, now).unwrap();
    assert_eq!(gatekeeper.admit(&one, now), Err(SkipReason::Pending));
    gatekeeper.started("a", now);
    assert_eq!(gatekeeper.admit(&one, now), Err(SkipReason::Running));
    assert!(gatekeeper.busy());
    gatekeeper.finished("a");
    assert!(!gatekeeper.busy());
}

#[test]
fn a_flood_of_one_automation_does_not_crowd_out_another() {
    let text = entry("audit-login", "{ event = \"user.sign-in-failed\" }", "")
        + &entry(
            "restart-nas",
            "{ event = \"service.status-changed\", services = [\"nas\"] }",
            "",
        );
    let sink = sink(&text);
    let failed = PortalEvent::visited(
        EventName::UserSignInFailed,
        &Visitor::default(),
        OffsetDateTime::UNIX_EPOCH,
    );
    for _ in 0..500 {
        sink.emit(failed.clone());
    }
    sink.emit(status_change("nas", "up", "down"));
    for _ in 0..500 {
        sink.emit(failed.clone());
    }
    let mut queued = Vec::new();
    while let Some(run) = sink.queue.take() {
        queued.push(run.automation.id);
    }
    assert_eq!(queued, vec!["audit-login", "restart-nas"]);
    let skips = sink.journal.runs(Some("audit-login"));
    assert_eq!(skips.len(), 1);
    assert_eq!(skips[0].seen.count, 999);
    assert_eq!(skips[0].result.reason.as_deref(), Some("pending"));
}

#[test]
fn emitting_does_not_wait_while_runs_are_going_on() {
    let sink = sink(&entry("a", "{ event = \"portal.started\" }", ""));
    for id in ["x", "y", "z", "w"] {
        sink.gatekeeper.started(id, Instant::now());
    }
    let began = Instant::now();
    for _ in 0..100 {
        sink.emit(PortalEvent::portal(
            EventName::PortalStarted,
            "",
            OffsetDateTime::UNIX_EPOCH,
        ));
    }
    assert!(began.elapsed() < Duration::from_millis(5 * 100));
    assert_eq!(sink.queue.len(), 1);
}

#[tokio::test]
async fn after_stopping_no_other_event_is_taken_and_settling_is_bounded() {
    let text = entry("stop", "{ event = \"portal.stopping\" }", "")
        + &entry("tick", "{ event = \"schedule\", cron = \"* * * * *\" }", "");
    let sink = sink(&text);
    sink.emit(PortalEvent::portal(
        EventName::PortalStopping,
        "",
        OffsetDateTime::UNIX_EPOCH,
    ));
    assert_eq!(sink.queue.len(), 1);
    sink.emit(
        PortalEvent::of(EventName::Schedule, OffsetDateTime::UNIX_EPOCH, &[]).aimed_at("tick"),
    );
    assert_eq!(sink.queue.len(), 1);
    let began = Instant::now();
    sink.settle(Duration::from_millis(300)).await;
    assert!(began.elapsed() < Duration::from_millis(600));
    assert!(sink.queue.is_empty());
    assert!(!sink.gatekeeper.busy());
}

#[test]
fn a_trigger_inside_the_cooldown_is_a_cooldown_even_while_the_first_run_goes_on() {
    let gatekeeper = Gatekeeper::default();
    let restart = automation(
        "restart",
        "{ event = \"portal.started\" }",
        "cooldown_seconds = 300",
    );
    let now = Instant::now();
    gatekeeper.admit(&restart, now).unwrap();
    assert_eq!(gatekeeper.admit(&restart, now), Err(SkipReason::Cooldown));
    gatekeeper.started("restart", now);
    assert_eq!(
        gatekeeper.admit(&restart, now + Duration::from_secs(5)),
        Err(SkipReason::Cooldown)
    );
}

#[test]
fn a_dropped_run_starts_no_cooldown() {
    let gatekeeper = Gatekeeper::default();
    let restart = automation(
        "restart",
        "{ event = \"portal.started\" }",
        "cooldown_seconds = 300",
    );
    let now = Instant::now();
    gatekeeper.admit(&restart, now).unwrap();
    gatekeeper.dequeued("restart");
    assert_eq!(gatekeeper.admit(&restart, now), Ok(()));
}

#[tokio::test]
async fn a_run_whose_task_panics_still_frees_its_automation() {
    let gatekeeper = std::sync::Arc::new(Gatekeeper::default());
    let one = automation("a", "{ event = \"portal.started\" }", "");
    gatekeeper.admit(&one, Instant::now()).unwrap();
    gatekeeper.started("a", Instant::now());
    let guard = crate::services::FinishGuard {
        gatekeeper: gatekeeper.clone(),
        automation: "a".into(),
    };
    let task = tokio::spawn(async move {
        let _guard = guard;
        panic!("a run went wrong");
    });
    assert!(task.await.is_err());
    assert!(!gatekeeper.busy());
}

#[test]
fn a_manual_run_during_shutdown_is_dropped_at_once() {
    let text = entry("a", "{ event = \"portal.started\" }", "")
        + &entry("stop", "{ event = \"portal.stopping\" }", "");
    let sink = sink(&text);
    sink.emit(PortalEvent::portal(
        EventName::PortalStopping,
        "",
        OffsetDateTime::UNIX_EPOCH,
    ));
    let queued = sink.queue.len();
    let id = sink.run_now(
        &automation("a", "{ event = \"portal.started\" }", ""),
        "admin",
    );
    assert_eq!(sink.queue.len(), queued);
    let runs = sink.journal.runs(Some("a"));
    assert_eq!(runs[0].id, id);
    assert_eq!(runs[0].result.reason.as_deref(), Some("dropped"));
}

#[tokio::test]
async fn settling_waits_for_a_killed_run_to_be_recorded_before_returning() {
    let sink = std::sync::Arc::new(sink(&entry("stop", "{ event = \"portal.stopping\" }", "")));
    sink.gatekeeper.started("stop", Instant::now());
    let finisher = sink.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(200)).await;
        finisher.gatekeeper.finished("stop");
    });
    let began = Instant::now();
    sink.settle(Duration::ZERO).await;
    assert!(began.elapsed() >= Duration::from_millis(200));
    assert!(!sink.gatekeeper.busy());
}

#[test]
fn active_runs_are_listed_newest_first_and_leave_when_removed() {
    let active = ActiveRuns::default();
    active.queued(&pending("a", 1, None), OffsetDateTime::UNIX_EPOCH);
    active.queued(&pending("b", 2, Some("admin")), OffsetDateTime::UNIX_EPOCH);
    active.started(1, vec!["--now".into()], OffsetDateTime::UNIX_EPOCH);
    let listed = active.matching(&RunFilter::default());
    assert_eq!(
        listed.iter().map(|run| run.run_id).collect::<Vec<_>>(),
        vec![2, 1]
    );
    assert!(listed[1].running());
    assert!(!listed[0].running());
    let by_text = RunFilter {
        text: Some("--NOW".into()),
        ..RunFilter::default()
    };
    assert_eq!(active.matching(&by_text).len(), 1);
    assert_eq!(active.of_automation("b").map(|run| run.run_id), Some(2));
    active.remove(1);
    assert_eq!(active.count(), 1);
}

#[test]
fn a_stop_is_requested_once() {
    let active = ActiveRuns::default();
    active.queued(&pending("a", 7, None), OffsetDateTime::UNIX_EPOCH);
    let control = active.control(7).unwrap();
    assert_eq!(active.request_stop(7, "alice"), Some(true));
    assert_eq!(active.request_stop(7, "bob"), Some(false));
    assert_eq!(active.request_stop(8, "bob"), None);
    assert!(control.stop_requested());
    assert_eq!(active.stopped_by(7).as_deref(), Some("alice"));
}

#[test]
fn an_admitted_run_is_active_and_a_dropped_one_is_not() {
    let sink = sink(&entry("a", "{ event = \"portal.started\" }", ""));
    let id = sink.run_now(
        &automation("a", "{ event = \"portal.started\" }", ""),
        "admin",
    );
    assert_eq!(sink.active.find(id).map(|run| run.running()), Some(false));
    let pending = sink.queue.take().unwrap();
    sink.forget(&pending);
    assert!(sink.active.find(id).is_none());
}
