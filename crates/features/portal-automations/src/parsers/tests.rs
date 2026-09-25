use jiff::tz::TimeZone;
use jiff::{Timestamp, Zoned};

use super::parse_cron;

fn at(text: &str) -> Timestamp {
    text.parse::<Zoned>().unwrap().timestamp()
}

fn zone(name: &str) -> TimeZone {
    TimeZone::get(name).unwrap()
}

fn fires(expression: &str, zone_name: &str, from: &str, count: usize) -> Vec<String> {
    let schedule = parse_cron(expression).unwrap();
    let zone = zone(zone_name);
    let mut cursor = at(from);
    let mut found = Vec::new();
    for _ in 0..count {
        cursor = schedule.next_after(cursor, &zone).unwrap();
        found.push(
            cursor
                .to_zoned(zone.clone())
                .strftime("%Y-%m-%d %H:%M %:z")
                .to_string(),
        );
    }
    found
}

#[test]
fn valid_expressions_parse() {
    for expression in [
        "* * * * *",
        "0 3 * * mon-fri",
        "*/15 0-6,22-23 1,15 jan-mar,DEC 0,7",
        "5/10 * * * *",
        "0 0 1-31/2 * *",
        "@hourly",
        "@daily",
        "@weekly",
        "@monthly",
    ] {
        assert!(parse_cron(expression).is_ok(), "{expression}");
    }
}

#[test]
fn every_error_names_its_field_and_its_range() {
    let cases = [
        ("61 * * * *", "minute must be 0 to 59"),
        ("* 24 * * *", "hour must be 0 to 23"),
        ("* * 0 * *", "day of month must be 1 to 31"),
        ("* * * 13 *", "month must be 1 to 12 or a name such as jan"),
        (
            "* * * * 8",
            "day of week must be 0 to 7 or a name such as sun",
        ),
        ("* * * * mox", "day of week must be 0 to 7"),
        ("*/0 * * * *", "minute step 0 must be a positive number"),
        ("10-5 * * * *", "minute range 10-5 goes backwards"),
        ("* * * *", "must have five fields"),
        ("@yearly", "must have five fields"),
    ];
    for (expression, expected) in cases {
        let error = parse_cron(expression).unwrap_err();
        assert!(error.contains(expected), "{expression}: {error}");
    }
}

#[test]
fn seven_and_zero_both_mean_sunday() {
    assert_eq!(
        parse_cron("0 0 * * 7").unwrap().weekdays,
        parse_cron("0 0 * * 0").unwrap().weekdays
    );
}

#[test]
fn weekdays_at_three_in_berlin_carry_the_offset_of_each_date() {
    assert_eq!(
        fires(
            "0 3 * * mon-fri",
            "Europe/Berlin",
            "2026-03-26T12:00:00+01:00[Europe/Berlin]",
            3
        ),
        vec![
            "2026-03-27 03:00 +01:00",
            "2026-03-30 03:00 +02:00",
            "2026-03-31 03:00 +02:00"
        ]
    );
}

#[test]
fn a_time_the_spring_change_skips_fires_once_at_the_jump() {
    assert_eq!(
        fires(
            "30 2 * * *",
            "Europe/Berlin",
            "2026-03-28T12:00:00+01:00[Europe/Berlin]",
            2
        ),
        vec!["2026-03-29 03:00 +02:00", "2026-03-30 02:30 +02:00"]
    );
}

#[test]
fn a_fixed_hour_fires_once_in_the_repeated_hour() {
    assert_eq!(
        fires(
            "30 1 * * *",
            "America/New_York",
            "2026-10-31T12:00:00-04:00[America/New_York]",
            3
        ),
        vec![
            "2026-11-01 01:30 -04:00",
            "2026-11-02 01:30 -05:00",
            "2026-11-03 01:30 -05:00"
        ]
    );
}

#[test]
fn a_wildcard_hour_keeps_firing_through_the_repeated_hour() {
    assert_eq!(
        fires(
            "*/30 * * * *",
            "America/New_York",
            "2026-11-01T00:45:00-04:00[America/New_York]",
            5
        ),
        vec![
            "2026-11-01 01:00 -04:00",
            "2026-11-01 01:30 -04:00",
            "2026-11-01 01:00 -05:00",
            "2026-11-01 01:30 -05:00",
            "2026-11-01 02:00 -05:00"
        ]
    );
}

#[test]
fn both_day_fields_restricted_match_either() {
    let found = fires("0 0 13 * fri", "UTC", "2026-02-01T00:00:00+00:00[UTC]", 6);
    assert_eq!(
        found,
        vec![
            "2026-02-06 00:00 +00:00",
            "2026-02-13 00:00 +00:00",
            "2026-02-20 00:00 +00:00",
            "2026-02-27 00:00 +00:00",
            "2026-03-06 00:00 +00:00",
            "2026-03-13 00:00 +00:00"
        ]
    );
}

#[test]
fn a_day_field_beginning_with_a_star_counts_as_unrestricted() {
    let found = fires("0 0 */2 * fri", "UTC", "2026-02-01T00:00:00+00:00[UTC]", 3);
    assert_eq!(
        found,
        vec![
            "2026-02-06 00:00 +00:00",
            "2026-02-13 00:00 +00:00",
            "2026-02-20 00:00 +00:00"
        ]
    );
}

#[test]
fn a_schedule_that_never_matches_never_fires() {
    let schedule = parse_cron("0 0 30 2 *").unwrap();
    assert_eq!(
        schedule.next_after(at("2026-01-01T00:00:00+00:00[UTC]"), &zone("UTC")),
        None
    );
}

#[test]
fn the_next_time_is_always_strictly_later() {
    let zone = zone("Europe/Berlin");
    for expression in ["* * * * *", "0 3 * * *", "*/5 * * * *", "30 2 * * *"] {
        let schedule = parse_cron(expression).unwrap();
        let mut cursor = at("2026-03-29T01:50:00+01:00[Europe/Berlin]");
        for _ in 0..200 {
            let next = schedule.next_after(cursor, &zone).unwrap();
            assert!(next > cursor, "{expression} at {cursor}");
            cursor = next;
        }
    }
}
