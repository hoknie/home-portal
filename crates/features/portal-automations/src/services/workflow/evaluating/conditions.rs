use serde_json::Value;

use super::frame::Frame;
use super::values::{render_text, render_value, text_of};
use crate::types::{Condition, Operator};

pub fn holds(condition: &Condition, frame: &Frame) -> Result<bool, String> {
    match condition {
        Condition::All(inner) => {
            for condition in inner {
                if !holds(condition, frame)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        Condition::Any(inner) => {
            for condition in inner {
                if holds(condition, frame)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        Condition::Compare {
            left,
            operator,
            right,
        } => {
            let left = render_value(left, frame)?;
            let right = match right {
                Some(right) => render_text(right, frame)?,
                None => String::new(),
            };
            Ok(compare(&left, *operator, &right))
        }
    }
}

pub fn compare(left: &Value, operator: Operator, right: &str) -> bool {
    let text = text_of(left);
    match operator {
        Operator::Equal => text == right,
        Operator::NotEqual => text != right,
        Operator::Less | Operator::LessOrEqual | Operator::Greater | Operator::GreaterOrEqual => {
            match (number(&text), number(right)) {
                (Some(left), Some(right)) => match operator {
                    Operator::Less => left < right,
                    Operator::LessOrEqual => left <= right,
                    Operator::Greater => left > right,
                    _ => left >= right,
                },
                _ => false,
            }
        }
        Operator::Contains => match left {
            Value::Array(items) => items.iter().any(|item| text_of(item) == right),
            _ => text.contains(right),
        },
        Operator::IsEmpty => empty(left),
        Operator::IsNotEmpty => !empty(left),
    }
}

fn number(text: &str) -> Option<f64> {
    text.trim()
        .parse::<f64>()
        .ok()
        .filter(|number| number.is_finite())
}

fn empty(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::String(text) => text.trim().is_empty(),
        Value::Array(items) => items.is_empty(),
        Value::Object(map) => map.is_empty(),
        _ => false,
    }
}
