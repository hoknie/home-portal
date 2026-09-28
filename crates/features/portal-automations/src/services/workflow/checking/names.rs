use portal_feature::FieldError;

use crate::helpers::OPEN;
use crate::types::{NumberSetting, RawNumber};

pub const LONGEST_NAME: usize = 63;
pub const NAME_RULE: &str =
    "must be 1 to 63 lowercase letters, digits and _, starting with a letter";

pub fn valid_name(name: &str) -> bool {
    let mut characters = name.chars();
    name.len() <= LONGEST_NAME
        && characters
            .next()
            .is_some_and(|first| first.is_ascii_lowercase())
        && characters.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

pub fn within(value: i64, minimum: i64, maximum: i64) -> bool {
    (minimum..=maximum).contains(&value)
}

pub fn between_rule(minimum: i64, maximum: i64) -> String {
    format!("must be {minimum} to {maximum}")
}

pub const NUMBER_RULE: &str = "must be a whole number or a template such as {{inputs.seconds}}";

pub fn number_setting(
    given: Option<&RawNumber>,
    (default, minimum, maximum): (Option<u64>, i64, i64),
    field: String,
    errors: &mut Vec<FieldError>,
) -> Option<NumberSetting> {
    let fixed = match given {
        None => return default.map(NumberSetting::Fixed),
        Some(RawNumber::Text(text)) if text.contains(OPEN) => {
            return Some(NumberSetting::Template(text.clone()));
        }
        Some(RawNumber::Text(text)) => text.trim().parse::<i64>().ok(),
        Some(RawNumber::Integer(number)) => Some(*number),
    };
    match fixed {
        Some(number) if within(number, minimum, maximum) => {
            Some(NumberSetting::Fixed(number as u64))
        }
        Some(_) => {
            errors.push(FieldError::new(field, between_rule(minimum, maximum)));
            None
        }
        None => {
            errors.push(FieldError::new(field, NUMBER_RULE));
            None
        }
    }
}
