use serde_json::Value as Json;
use toml_edit::{Array, InlineTable, Value};

pub fn toml_of(json: &Json) -> Option<Value> {
    match json {
        Json::Null => None,
        Json::Bool(flag) => Some(Value::from(*flag)),
        Json::Number(number) => number
            .as_i64()
            .map(Value::from)
            .or_else(|| number.as_f64().map(Value::from)),
        Json::String(text) => Some(Value::from(text.as_str())),
        Json::Array(items) => {
            let array: Array = items.iter().filter_map(toml_of).collect();
            Some(Value::Array(array))
        }
        Json::Object(fields) => {
            let mut table = InlineTable::new();
            for (key, field) in fields {
                if let Some(value) = toml_of(field) {
                    table.insert(key, value);
                }
            }
            Some(Value::InlineTable(table))
        }
    }
}

pub fn unique_id(kind: &str, taken: &[String]) -> String {
    let base: String = kind
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let base = if base.starts_with(|character: char| character.is_ascii_lowercase()) {
        base
    } else {
        format!("widget-{base}")
    };
    if !taken.contains(&base) {
        return base;
    }
    (2..)
        .map(|number| format!("{base}-{number}"))
        .find(|candidate| !taken.contains(candidate))
        .unwrap_or(base)
}
