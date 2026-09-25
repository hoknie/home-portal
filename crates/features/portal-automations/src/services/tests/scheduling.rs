use jiff::tz::TimeZone;
use jiff::{Timestamp, Zoned};

use super::support::automations;
use crate::services::ScheduleBook;

fn at(text: &str) -> Timestamp {
    text.parse::<Zoned>().unwrap().timestamp()
}

fn daily_at_three() -> String {
    super::support::entry(
        "backup",
        "{ event = \"schedule\", cron = \"0 3 * * *\" }",
        "",
    )
}

#[test]
fn a_tick_that_wakes_late_still_fires_the_run_exactly_once() {
    let list = automations(&daily_at_three());
    let zone = TimeZone::get("Europe/Berlin").unwrap();
    let mut book = ScheduleBook::default();
    assert!(
        book.due(&list, &zone, at("2026-05-01T02:59:59+02:00[Europe/Berlin]"))
            .is_empty()
    );
    let fired = book.due(
        &list,
        &zone,
        at("2026-05-01T03:00:00.4+02:00[Europe/Berlin]"),
    );
    assert_eq!(fired.len(), 1);
    assert_eq!(fired[0].2, at("2026-05-01T03:00:00+02:00[Europe/Berlin]"));
    assert!(
        book.due(
            &list,
            &zone,
            at("2026-05-01T03:00:01.9+02:00[Europe/Berlin]")
        )
        .is_empty()
    );
}

#[test]
fn an_edited_schedule_fires_at_its_new_time() {
    let zone = TimeZone::UTC;
    let mut book = ScheduleBook::default();
    let old = automations(&daily_at_three());
    book.due(&old, &zone, at("2026-05-01T10:00:00+00:00[UTC]"));
    let edited = automations(&super::support::entry(
        "backup",
        "{ event = \"schedule\", cron = \"1 10 * * *\" }",
        "",
    ));
    assert!(
        book.due(&edited, &zone, at("2026-05-01T10:00:30+00:00[UTC]"))
            .is_empty()
    );
    assert_eq!(book.earliest(), Some(at("2026-05-01T10:01:00+00:00[UTC]")));
    assert_eq!(
        book.due(&edited, &zone, at("2026-05-01T10:01:01+00:00[UTC]"))
            .len(),
        1
    );
}

#[test]
fn the_new_york_autumn_night_fires_a_fixed_hour_once() {
    let list = automations(&super::support::entry(
        "night",
        "{ event = \"schedule\", cron = \"30 1 * * *\" }",
        "",
    ));
    let zone = TimeZone::get("America/New_York").unwrap();
    let mut book = ScheduleBook::default();
    let mut cursor = at("2026-11-01T00:00:00-04:00[America/New_York]");
    book.due(&list, &zone, cursor);
    let mut fired = 0;
    for _ in 0..(3 * 3600) {
        cursor = cursor
            .checked_add(jiff::SignedDuration::from_secs(1))
            .unwrap();
        fired += book.due(&list, &zone, cursor).len();
    }
    assert_eq!(fired, 1);
}

#[test]
fn a_missed_time_is_not_caught_up() {
    let list = automations(&daily_at_three());
    let zone = TimeZone::UTC;
    let mut book = ScheduleBook::default();
    book.due(&list, &zone, at("2026-05-01T02:00:00+00:00[UTC]"));
    assert!(
        book.due(&list, &zone, at("2026-05-01T09:00:00+00:00[UTC]"))
            .is_empty()
    );
}
