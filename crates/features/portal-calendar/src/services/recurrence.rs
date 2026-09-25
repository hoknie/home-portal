use std::collections::BTreeMap;

use time::{Duration, OffsetDateTime, Weekday};

use crate::types::{CalendarEvent, RawEvent, Repeats};

pub const SUPPORTED_KEYS: [&str; 5] = ["FREQ", "INTERVAL", "COUNT", "UNTIL", "BYDAY"];
pub const WEEKDAYS: [(&str, Weekday); 7] = [
    ("MO", Weekday::Monday),
    ("TU", Weekday::Tuesday),
    ("WE", Weekday::Wednesday),
    ("TH", Weekday::Thursday),
    ("FR", Weekday::Friday),
    ("SA", Weekday::Saturday),
    ("SU", Weekday::Sunday),
];

pub fn expand(
    event: &RawEvent,
    calendar: Option<&str>,
    from: OffsetDateTime,
    until: OffsetDateTime,
) -> Vec<CalendarEvent> {
    let Some(rule) = &event.rule else {
        return occurrence(event, calendar, event.start, Repeats::Never)
            .filter(|_| event.start >= from && event.start < until)
            .into_iter()
            .collect();
    };
    let parts = parts_of(rule);
    if !understood(&parts) {
        return occurrence(event, calendar, event.start, Repeats::Unsupported)
            .into_iter()
            .collect();
    }
    let step = parts
        .get("INTERVAL")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(1)
        .max(1);
    let count = parts
        .get("COUNT")
        .and_then(|value| value.parse::<usize>().ok());
    let ends = parts
        .get("UNTIL")
        .and_then(|value| super::moments::parse_moment(value, event.start.offset()))
        .map(|(moment, _)| moment);
    let days = weekdays(parts.get("BYDAY").map(String::as_str));
    let mut moments = Vec::new();
    let mut produced = 0usize;
    let mut moment = event.start;
    let mut guard = 0;
    while moment < until && guard < 5_000 {
        guard += 1;
        if ends.is_some_and(|ends| moment > ends) || count.is_some_and(|count| produced >= count) {
            break;
        }
        let wanted = days.is_empty() || days.contains(&moment.weekday());
        if wanted {
            produced += 1;
            if moment >= from && !event.excluded.contains(&moment) {
                moments.push(moment);
            }
        }
        moment = next(
            moment,
            parts.get("FREQ").map(String::as_str).unwrap_or("DAILY"),
            step,
            !days.is_empty(),
        );
    }
    moments
        .into_iter()
        .filter_map(|moment| occurrence(event, calendar, moment, Repeats::Expanded))
        .collect()
}

fn occurrence(
    event: &RawEvent,
    calendar: Option<&str>,
    start: OffsetDateTime,
    repeats: Repeats,
) -> Option<CalendarEvent> {
    Some(CalendarEvent {
        summary: event.summary.clone(),
        start,
        end: event.duration.map(|duration| start + duration),
        all_day: event.all_day,
        location: event.location.clone(),
        calendar: calendar.map(str::to_string),
        repeats,
    })
}

fn parts_of(rule: &str) -> BTreeMap<String, String> {
    rule.split(';')
        .filter_map(|part| part.split_once('='))
        .map(|(key, value)| (key.trim().to_ascii_uppercase(), value.trim().to_string()))
        .collect()
}

fn understood(parts: &BTreeMap<String, String>) -> bool {
    let frequency = parts.get("FREQ").map(String::as_str).unwrap_or_default();
    let known_frequency = matches!(frequency, "DAILY" | "WEEKLY" | "MONTHLY" | "YEARLY");
    let known_keys = parts
        .keys()
        .all(|key| SUPPORTED_KEYS.contains(&key.as_str()));
    let plain_days = parts
        .get("BYDAY")
        .map(|days| {
            days.split(',')
                .all(|day| WEEKDAYS.iter().any(|(name, _)| *name == day.trim()))
        })
        .unwrap_or(true);
    known_frequency && known_keys && plain_days
}

fn weekdays(byday: Option<&str>) -> Vec<Weekday> {
    byday
        .map(|days| {
            days.split(',')
                .filter_map(|day| {
                    WEEKDAYS
                        .iter()
                        .find(|(name, _)| *name == day.trim())
                        .map(|(_, weekday)| *weekday)
                })
                .collect()
        })
        .unwrap_or_default()
}

fn next(moment: OffsetDateTime, frequency: &str, step: i64, by_weekday: bool) -> OffsetDateTime {
    match frequency {
        "DAILY" => moment + Duration::days(step),
        "WEEKLY" if by_weekday => moment + Duration::days(1),
        "WEEKLY" => moment + Duration::weeks(step),
        "MONTHLY" => add_months(moment, step),
        _ => add_months(moment, step * 12),
    }
}

fn add_months(moment: OffsetDateTime, months: i64) -> OffsetDateTime {
    let mut year = moment.year();
    let mut month = i64::from(moment.month() as u8) + months;
    while month > 12 {
        month -= 12;
        year += 1;
    }
    while month < 1 {
        month += 12;
        year -= 1;
    }
    let month = time::Month::try_from(month as u8).unwrap_or(moment.month());
    let day = moment.day().min(days_in(year, month));
    moment
        .replace_year(year)
        .and_then(|moment| moment.replace_month(month))
        .and_then(|moment| moment.replace_day(day))
        .unwrap_or(moment)
}

fn days_in(year: i32, month: time::Month) -> u8 {
    time::util::days_in_month(month, year)
}
