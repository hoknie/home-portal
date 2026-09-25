use crate::types::{Schedule, Wildcards};

pub const MONTHS: [&str; 12] = [
    "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
];
pub const WEEKDAYS: [&str; 7] = ["sun", "mon", "tue", "wed", "thu", "fri", "sat"];

struct Field {
    label: &'static str,
    low: u8,
    high: u8,
    names: &'static [&'static str],
}

const FIELDS: [Field; 5] = [
    Field {
        label: "minute",
        low: 0,
        high: 59,
        names: &[],
    },
    Field {
        label: "hour",
        low: 0,
        high: 23,
        names: &[],
    },
    Field {
        label: "day of month",
        low: 1,
        high: 31,
        names: &[],
    },
    Field {
        label: "month",
        low: 1,
        high: 12,
        names: &MONTHS,
    },
    Field {
        label: "day of week",
        low: 0,
        high: 7,
        names: &WEEKDAYS,
    },
];

pub fn parse_cron(expression: &str) -> Result<Schedule, String> {
    let expanded = match expression.trim() {
        "@hourly" => "0 * * * *",
        "@daily" => "0 0 * * *",
        "@weekly" => "0 0 * * 0",
        "@monthly" => "0 0 1 * *",
        other => other,
    };
    let parts: Vec<&str> = expanded.split_whitespace().collect();
    if parts.len() != FIELDS.len() {
        return Err(format!(
            "must have five fields (minute, hour, day of month, month, day of week) or be @hourly, @daily, @weekly or @monthly, not {}",
            parts.len()
        ));
    }
    let mut sets = [0u64; 5];
    for (index, (part, field)) in parts.iter().zip(FIELDS.iter()).enumerate() {
        sets[index] = parse_field(part, field)?;
    }
    let weekdays = sets[4] | (sets[4] >> 7 & 1);
    Ok(Schedule {
        minutes: sets[0],
        hours: sets[1],
        days: sets[2],
        months: sets[3],
        weekdays: weekdays & 0x7f,
        wildcards: Wildcards {
            hour: parts[1].starts_with('*'),
            day: parts[2].starts_with('*'),
            weekday: parts[4].starts_with('*'),
        },
    })
}

fn parse_field(text: &str, field: &Field) -> Result<u64, String> {
    let mut set = 0u64;
    for item in text.split(',') {
        let (range, step) = match item.split_once('/') {
            Some((range, step)) => (range, Some(step)),
            None => (item, None),
        };
        let (low, high) = if range == "*" {
            (field.low, field.high)
        } else if let Some((low, high)) = range.split_once('-') {
            (value_of(low, field)?, value_of(high, field)?)
        } else {
            let single = value_of(range, field)?;
            let high = if step.is_some() { field.high } else { single };
            (single, high)
        };
        if low > high {
            return Err(format!("{} range {range} goes backwards", field.label));
        }
        let step = match step {
            None => 1,
            Some(step) => match step.parse::<u8>() {
                Ok(step) if step > 0 => step,
                _ => {
                    return Err(format!(
                        "{} step {step} must be a positive number",
                        field.label
                    ));
                }
            },
        };
        let mut value = low;
        while value <= high {
            set |= 1 << value;
            value = match value.checked_add(step) {
                Some(next) => next,
                None => break,
            };
        }
    }
    Ok(set)
}

fn value_of(text: &str, field: &Field) -> Result<u8, String> {
    let lowered = text.to_ascii_lowercase();
    if let Some(position) = field.names.iter().position(|name| *name == lowered) {
        let first = if field.names.len() == MONTHS.len() {
            1
        } else {
            0
        };
        return Ok(position as u8 + first);
    }
    match text.parse::<u8>() {
        Ok(value) if (field.low..=field.high).contains(&value) => Ok(value),
        _ => Err(format!(
            "{} must be {} to {}{}",
            field.label,
            field.low,
            field.high,
            if field.names.is_empty() {
                String::new()
            } else {
                format!(" or a name such as {}", field.names[0])
            }
        )),
    }
}
