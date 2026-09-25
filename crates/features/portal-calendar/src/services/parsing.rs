use std::io::BufReader;

use ical::IcalParser;
use ical::property::Property;
use time::{Duration, UtcOffset};

use super::moments::parse_moment;
use crate::types::RawEvent;

pub struct CalendarFeed {
    pub name: Option<String>,
    pub events: Vec<RawEvent>,
}

pub fn parse_feed(text: &str, offset: UtcOffset) -> Result<CalendarFeed, String> {
    let mut name = None;
    let mut events = Vec::new();
    for calendar in IcalParser::new(BufReader::new(text.as_bytes())) {
        let calendar =
            calendar.map_err(|error| format!("the calendar could not be read: {error}"))?;
        if name.is_none() {
            name = value_of(&calendar.properties, "X-WR-CALNAME");
        }
        for event in calendar.events {
            if let Some(raw) = raw_event(&event.properties, offset) {
                events.push(raw);
            }
        }
    }
    Ok(CalendarFeed { name, events })
}

fn raw_event(properties: &[Property], offset: UtcOffset) -> Option<RawEvent> {
    let (start, all_day) = parse_moment(&value_of(properties, "DTSTART")?, offset)?;
    let end = value_of(properties, "DTEND").and_then(|text| parse_moment(&text, offset));
    let duration = match end {
        Some((moment, _)) => Some(moment - start),
        None if all_day => Some(Duration::days(1)),
        None => None,
    };
    Some(RawEvent {
        summary: value_of(properties, "SUMMARY").unwrap_or_default(),
        start,
        duration,
        all_day,
        location: value_of(properties, "LOCATION"),
        rule: value_of(properties, "RRULE"),
        excluded: values_of(properties, "EXDATE")
            .into_iter()
            .flat_map(|text| text.split(',').map(str::to_string).collect::<Vec<_>>())
            .filter_map(|text| parse_moment(&text, offset).map(|(moment, _)| moment))
            .collect(),
    })
}

fn value_of(properties: &[Property], name: &str) -> Option<String> {
    properties
        .iter()
        .find(|property| property.name == name)
        .and_then(|property| property.value.clone())
}

fn values_of(properties: &[Property], name: &str) -> Vec<String> {
    properties
        .iter()
        .filter(|property| property.name == name)
        .filter_map(|property| property.value.clone())
        .collect()
}
