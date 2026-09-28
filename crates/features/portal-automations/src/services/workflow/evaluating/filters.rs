use std::cmp::Ordering;

use serde_json::Value;

use super::frame::walk;
use super::values::text_of;
use crate::types::{FilterCall, ValueType, filter_named};

pub fn apply_chain(value: Value, filters: &[FilterCall]) -> Result<Value, String> {
    filters.iter().try_fold(value, apply_filter)
}

pub fn apply_filter(value: Value, call: &FilterCall) -> Result<Value, String> {
    let description =
        filter_named(&call.name).ok_or_else(|| format!("{} is not a filter", call.name))?;
    if value.is_null() && !matches!(call.name.as_str(), "default" | "json") {
        return Ok(Value::Null);
    }
    let got = ValueType::of(&value);
    if let (true, Value::Array(items)) = (description.element && !description.takes(got), &value) {
        let mut mapped = Vec::with_capacity(items.len());
        for (index, item) in items.iter().enumerate() {
            let kind = ValueType::of(item);
            if !item.is_null() && !description.takes(kind) {
                return Err(format!(
                    "{} takes {} and got {} at position {index}",
                    call.name,
                    description.accepted(),
                    kind.described()
                ));
            }
            mapped.push(apply_filter(item.clone(), call)?);
        }
        return Ok(Value::Array(mapped));
    }
    if !description.takes(got) {
        return Err(format!(
            "{} takes {} and got {}",
            call.name,
            description.accepted(),
            got.described()
        ));
    }
    let argument = |index: usize| call.arguments.get(index).cloned().unwrap_or(Value::Null);
    let text_argument = |index: usize| text_of(&argument(index));
    match (call.name.as_str(), value) {
        ("upper", Value::String(text)) => Ok(text.to_uppercase().into()),
        ("lower", Value::String(text)) => Ok(text.to_lowercase().into()),
        ("trim", Value::String(text)) => Ok(text.trim().into()),
        ("replace", Value::String(text)) => {
            Ok(text.replace(&text_argument(0), &text_argument(1)).into())
        }
        ("split", Value::String(text)) => Ok(Value::Array(
            text.split(text_argument(0).as_str())
                .map(|part| Value::String(part.to_string()))
                .collect(),
        )),
        ("slice", Value::String(text)) => {
            let characters: Vec<char> = text.chars().collect();
            let (start, end) = bounds(characters.len(), &argument(0), &argument(1));
            Ok(characters[start..end].iter().collect::<String>().into())
        }
        ("slice", Value::Array(items)) => {
            let (start, end) = bounds(items.len(), &argument(0), &argument(1));
            Ok(Value::Array(items[start..end].to_vec()))
        }
        ("starts_with", Value::String(text)) => Ok(text.starts_with(&text_argument(0)).into()),
        ("contains", Value::String(text)) => Ok(text.contains(&text_argument(0)).into()),
        ("contains", Value::Array(items)) => {
            let wanted = argument(0);
            Ok(items
                .iter()
                .any(|item| *item == wanted || text_of(item) == text_of(&wanted))
                .into())
        }
        ("length", Value::String(text)) => Ok(text.chars().count().into()),
        ("length", Value::Array(items)) => Ok(items.len().into()),
        ("length", Value::Object(map)) => Ok(map.len().into()),
        ("default", value) => Ok(if empty(&value) { argument(0) } else { value }),
        ("number", Value::String(text)) => text
            .trim()
            .parse::<f64>()
            .ok()
            .filter(|number| number.is_finite())
            .map(number_value)
            .ok_or_else(|| format!("number cannot read \"{text}\" as a number")),
        ("number", value @ Value::Number(_)) => Ok(value),
        ("round", Value::Number(number)) => {
            let digits = argument(0).as_i64().unwrap_or(0).clamp(0, 10) as i32;
            let scale = 10f64.powi(digits);
            Ok(number_value(
                (number.as_f64().unwrap_or(0.0) * scale).round() / scale,
            ))
        }
        ("floor", Value::Number(number)) => {
            Ok(number_value(number.as_f64().unwrap_or(0.0).floor()))
        }
        ("ceil", Value::Number(number)) => Ok(number_value(number.as_f64().unwrap_or(0.0).ceil())),
        ("abs", Value::Number(number)) => Ok(number_value(number.as_f64().unwrap_or(0.0).abs())),
        ("first", Value::Array(items)) => Ok(items.into_iter().next().unwrap_or(Value::Null)),
        ("last", Value::Array(items)) => Ok(items.into_iter().last().unwrap_or(Value::Null)),
        ("join", Value::Array(items)) => Ok(items
            .iter()
            .map(text_of)
            .collect::<Vec<_>>()
            .join(&text_argument(0))
            .into()),
        ("sort", Value::Array(mut items)) => {
            items.sort_by(order_of);
            Ok(Value::Array(items))
        }
        ("reverse", Value::Array(mut items)) => {
            items.reverse();
            Ok(Value::Array(items))
        }
        ("unique", Value::Array(items)) => {
            let mut kept: Vec<Value> = Vec::new();
            for item in items {
                if !kept.contains(&item) {
                    kept.push(item);
                }
            }
            Ok(Value::Array(kept))
        }
        ("pluck", Value::Array(items)) => {
            let key = text_argument(0);
            Ok(Value::Array(
                items.into_iter().map(|item| at_key(item, &key)).collect(),
            ))
        }
        ("sum" | "min" | "max", Value::Array(items)) => aggregate(&call.name, &items),
        ("keys", Value::Object(map)) => Ok(Value::Array(
            map.keys().cloned().map(Value::String).collect(),
        )),
        ("values", Value::Object(map)) => Ok(Value::Array(
            map.into_iter().map(|(_, value)| value).collect(),
        )),
        ("get", value) => Ok(at_key(value, &text_argument(0))),
        ("json", value) => Ok(Value::String(value.to_string())),
        ("parse", Value::String(text)) => serde_json::from_str(&text)
            .map_err(|_| "parse cannot read the text as JSON".to_string()),
        (name, _) => Err(format!("{name} is not a filter")),
    }
}

pub fn at_key(value: Value, key: &str) -> Value {
    let path: Vec<&str> = key.split('.').filter(|part| !part.is_empty()).collect();
    walk(value, &path)
}

pub fn number_value(number: f64) -> Value {
    if number.fract() == 0.0 && number.abs() < 9.0e15 {
        Value::from(number as i64)
    } else {
        serde_json::Number::from_f64(number).map_or(Value::Null, Value::Number)
    }
}

pub fn order_of(left: &Value, right: &Value) -> Ordering {
    match (left, right) {
        (Value::Number(left), Value::Number(right)) => left
            .as_f64()
            .partial_cmp(&right.as_f64())
            .unwrap_or(Ordering::Equal),
        _ => rank(left)
            .cmp(&rank(right))
            .then_with(|| text_of(left).cmp(&text_of(right))),
    }
}

fn rank(value: &Value) -> u8 {
    match value {
        Value::Null => 0,
        Value::Bool(_) => 1,
        Value::Number(_) => 2,
        Value::String(_) => 3,
        Value::Array(_) => 4,
        Value::Object(_) => 5,
    }
}

fn empty(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::String(text) => text.is_empty(),
        _ => false,
    }
}

fn bounds(length: usize, start: &Value, end: &Value) -> (usize, usize) {
    let place = |value: &Value, fallback: usize| match value.as_i64() {
        None => fallback,
        Some(index) if index < 0 => length.saturating_sub(index.unsigned_abs() as usize),
        Some(index) => (index as usize).min(length),
    };
    let start = place(start, 0);
    let end = place(end, length);
    (start.min(end), end)
}

fn aggregate(name: &str, items: &[Value]) -> Result<Value, String> {
    let mut numbers = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        match item.as_f64() {
            Some(number) => numbers.push(number),
            None => {
                return Err(format!(
                    "{name} takes a list of numbers and got {} at position {index}",
                    ValueType::of(item).described()
                ));
            }
        }
    }
    let result = match name {
        "sum" => Some(numbers.iter().sum()),
        "min" => numbers.iter().copied().reduce(f64::min),
        _ => numbers.iter().copied().reduce(f64::max),
    };
    Ok(result.map_or(Value::Null, number_value))
}
