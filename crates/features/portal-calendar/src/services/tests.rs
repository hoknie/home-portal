use time::macros::datetime;
use time::{Duration, OffsetDateTime, UtcOffset};

use super::moments::parse_moment;
use super::{expand, offset_of, parse_feed};
use crate::types::Repeats;

const FEED: &str = "BEGIN:VCALENDAR\r\nX-WR-CALNAME:Family\r\nBEGIN:VEVENT\r\nSUMMARY:Dentist\r\nDTSTART:20260923T090000Z\r\nDTEND:20260923T100000Z\r\nLOCATION:Riga\r\nEND:VEVENT\r\nBEGIN:VEVENT\r\nSUMMARY:Holiday\r\nDTSTART;VALUE=DATE:20260924\r\nEND:VEVENT\r\nBEGIN:VEVENT\r\nSUMMARY:Far away\r\nDTSTART:20261223T090000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

fn window() -> (OffsetDateTime, OffsetDateTime) {
    let now = datetime!(2026-09-22 08:00 UTC);
    (now, now + Duration::days(7))
}

fn events_of(text: &str) -> Vec<crate::types::CalendarEvent> {
    let (from, until) = window();
    let feed = parse_feed(text, UtcOffset::UTC).unwrap();
    let mut events: Vec<_> = feed
        .events
        .iter()
        .flat_map(|event| expand(event, feed.name.as_deref(), from, until))
        .collect();
    events.sort_by_key(|event| event.start);
    events
}

#[test]
fn the_events_in_the_window_are_listed_in_order_with_their_details() {
    let events = events_of(FEED);
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].summary, "Dentist");
    assert_eq!(events[0].start, datetime!(2026-09-23 09:00 UTC));
    assert_eq!(events[0].end, Some(datetime!(2026-09-23 10:00 UTC)));
    assert_eq!(events[0].location.as_deref(), Some("Riga"));
    assert_eq!(events[0].calendar.as_deref(), Some("Family"));
    assert!(!events[0].all_day);
    assert_eq!(events[0].repeats, Repeats::Never);
    assert!(events[1].all_day);
    assert_eq!(events[1].summary, "Holiday");
    assert_eq!(events[1].end, Some(datetime!(2026-09-25 00:00 UTC)));
}

#[test]
fn a_weekly_meeting_appears_on_each_of_its_days() {
    let feed = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nSUMMARY:Standup\r\nDTSTART:20260907T080000Z\r\nDTEND:20260907T081500Z\r\nRRULE:FREQ=WEEKLY;BYDAY=MO\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
    let events = events_of(feed);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].start, datetime!(2026-09-28 08:00 UTC));
    assert_eq!(events[0].repeats, Repeats::Expanded);
    let (from, until) = window();
    let feed = parse_feed(feed, UtcOffset::UTC).unwrap();
    let fortnight = expand(&feed.events[0], None, from, until + Duration::days(7));
    assert_eq!(fortnight.len(), 2);
    assert_eq!(fortnight[1].start, datetime!(2026-10-05 08:00 UTC));
}

#[test]
fn a_daily_repeat_honours_interval_count_and_exclusions() {
    let feed = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nSUMMARY:Pills\r\nDTSTART:20260922T090000Z\r\nRRULE:FREQ=DAILY;INTERVAL=2;COUNT=3\r\nEXDATE:20260924T090000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
    let starts: Vec<OffsetDateTime> = events_of(feed)
        .into_iter()
        .map(|event| event.start)
        .collect();
    assert_eq!(
        starts,
        vec![
            datetime!(2026-09-22 09:00 UTC),
            datetime!(2026-09-26 09:00 UTC)
        ]
    );
}

#[test]
fn a_repeat_ends_at_its_until() {
    let feed = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nSUMMARY:Sprint\r\nDTSTART:20260922T090000Z\r\nRRULE:FREQ=DAILY;UNTIL=20260924T090000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
    assert_eq!(events_of(feed).len(), 3);
}

#[test]
fn a_monthly_and_a_yearly_repeat_land_on_the_same_day_of_the_month() {
    let feed = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nSUMMARY:Rent\r\nDTSTART:20260825T090000Z\r\nRRULE:FREQ=MONTHLY\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
    let events = events_of(feed);
    assert_eq!(events[0].start, datetime!(2026-09-25 09:00 UTC));
    let yearly = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nSUMMARY:Birthday\r\nDTSTART:20200924T000000Z\r\nRRULE:FREQ=YEARLY\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
    assert_eq!(events_of(yearly)[0].start, datetime!(2026-09-24 00:00 UTC));
}

#[test]
fn a_repeat_the_portal_cannot_expand_is_shown_once_and_marked() {
    let feed = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nSUMMARY:Board\r\nDTSTART:20260901T090000Z\r\nRRULE:FREQ=WEEKLY;BYSETPOS=-1;BYDAY=MO\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
    let events = events_of(feed);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].repeats, Repeats::Unsupported);
    assert_eq!(events[0].start, datetime!(2026-09-01 09:00 UTC));
}

#[test]
fn a_moment_is_read_as_utc_a_floating_time_or_a_whole_day() {
    let offset = UtcOffset::from_hms(3, 0, 0).unwrap();
    assert_eq!(
        parse_moment("20260922T090000Z", offset).unwrap().0,
        datetime!(2026-09-22 09:00 UTC)
    );
    let (floating, all_day) = parse_moment("20260922T090000", offset).unwrap();
    assert_eq!(floating, datetime!(2026-09-22 09:00 +3));
    assert!(!all_day);
    let (day, all_day) = parse_moment("20260922", offset).unwrap();
    assert_eq!(day, datetime!(2026-09-22 00:00 +3));
    assert!(all_day);
    assert!(parse_moment("nonsense", offset).is_none());
}

#[test]
fn a_timezone_is_utc_or_an_offset_and_nothing_else() {
    assert_eq!(offset_of(None).unwrap(), UtcOffset::UTC);
    assert_eq!(offset_of(Some("UTC")).unwrap(), UtcOffset::UTC);
    assert_eq!(
        offset_of(Some("+03:00")).unwrap(),
        UtcOffset::from_hms(3, 0, 0).unwrap()
    );
    assert_eq!(
        offset_of(Some("-05:30")).unwrap(),
        UtcOffset::from_hms(-5, -30, 0).unwrap()
    );
    let refused = offset_of(Some("Europe/Riga")).unwrap_err();
    assert!(refused.contains("+03:00"), "{refused}");
}
