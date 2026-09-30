use serde_json::Value;

pub fn walk(value: Value, path: &[&str]) -> Value {
    path.iter().fold(value, |current, part| match current {
        Value::Object(mut map) => map
            .remove(*part)
            .or_else(|| {
                let lowered = part.to_ascii_lowercase();
                map.into_iter()
                    .find(|(key, _)| key.to_ascii_lowercase() == lowered)
                    .map(|(_, value)| value)
            })
            .unwrap_or(Value::Null),
        Value::Array(mut items) => part
            .parse::<usize>()
            .ok()
            .filter(|index| *index < items.len())
            .map(|index| items.swap_remove(index))
            .unwrap_or(Value::Null),
        _ => Value::Null,
    })
}

pub fn text_of(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}
