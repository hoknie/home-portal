use jiff::tz::TimeZone;
use jiff::{Timestamp, Zoned};
use portal_feature::{EventName, EventSink, PortalEvent};
use time::OffsetDateTime;
use toml_edit::DocumentMut;

use super::support::{automations, entry, sink};
use crate::services::ScheduleBook;
use crate::types::SkipReason;

const OFF: &str = "[modules]\nautomations = false\nwebhooks = false\n\n";

fn started() -> PortalEvent {
    PortalEvent::portal(EventName::PortalStarted, "", OffsetDateTime::UNIX_EPOCH)
}

fn at(text: &str) -> Timestamp {
    text.parse::<Zoned>().unwrap().timestamp()
}

#[test]
fn an_event_while_the_module_is_off_admits_nothing_and_journals_nothing() {
    let text = entry("hello", "{ event = \"portal.started\" }", "");
    let sink = sink(&format!("{OFF}{text}"));
    sink.emit(started());
    assert!(sink.queue.is_empty());
    assert!(sink.journal.snapshot().is_empty());
}

#[test]
fn portal_stopping_while_off_runs_nothing_but_still_stops_the_sink() {
    let text = entry("bye", "{ event = \"portal.stopping\" }", "");
    let sink = sink(&format!("{OFF}{text}"));
    sink.emit(PortalEvent::portal(
        EventName::PortalStopping,
        "",
        OffsetDateTime::UNIX_EPOCH,
    ));
    assert!(sink.queue.is_empty());
    assert!(sink.journal.snapshot().is_empty());
    assert!(sink.stopping());
}

#[test]
fn switching_off_journals_queued_runs_as_removed_and_leaves_started_ones_alone() {
    let text = format!(
        "{}{}",
        entry("first", "{ event = \"portal.started\" }", ""),
        entry("second", "{ event = \"portal.started\" }", "")
    );
    let sink = sink(&text);
    sink.emit(started());
    let started_run = sink.queue.take().unwrap();
    sink.cache
        .refresh(&format!("{OFF}{text}").parse::<DocumentMut>().unwrap());
    sink.drain_as_removed();
    let journal = sink.journal.snapshot();
    assert_eq!(journal.len(), 1);
    assert_eq!(journal[0].automation, "second");
    assert_eq!(
        journal[0].result.reason.as_deref(),
        Some(SkipReason::Removed.name())
    );
    assert!(sink.journal.find(started_run.run_id).is_none());
    assert!(!sink.cache.runnable("first"));
}

#[test]
fn a_schedule_missed_while_off_is_not_replayed_when_switched_back_on() {
    let text = entry(
        "backup",
        "{ event = \"schedule\", cron = \"0 3 * * *\" }",
        "",
    );
    let sink = sink(&format!("{OFF}{text}"));
    let list = automations(&text);
    let zone = TimeZone::get("Europe/Berlin").unwrap();
    let mut book = ScheduleBook::default();
    book.due(&list, &zone, at("2026-05-01T02:00:00+02:00[Europe/Berlin]"));
    let fired = book.due(
        &list,
        &zone,
        at("2026-05-01T03:00:00.5+02:00[Europe/Berlin]"),
    );
    assert_eq!(fired.len(), 1);
    for (id, _, _) in fired {
        sink.emit(
            PortalEvent::of(EventName::Schedule, OffsetDateTime::UNIX_EPOCH, &[]).aimed_at(id),
        );
    }
    assert!(sink.journal.snapshot().is_empty());
    assert!(sink.queue.is_empty());
    sink.cache.refresh(&text.parse::<DocumentMut>().unwrap());
    assert!(
        book.due(&list, &zone, at("2026-05-01T09:00:00+02:00[Europe/Berlin]"))
            .is_empty()
    );
    let next = book.due(
        &list,
        &zone,
        at("2026-05-02T03:00:00.5+02:00[Europe/Berlin]"),
    );
    assert_eq!(next.len(), 1);
    for (id, _, _) in next {
        sink.emit(
            PortalEvent::of(EventName::Schedule, OffsetDateTime::UNIX_EPOCH, &[]).aimed_at(id),
        );
    }
    assert_eq!(sink.queue.len(), 1);
}
