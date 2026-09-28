use serde_json::Value;

pub const LARGEST_SHAPE: usize = 16 * 1024;
pub const SHAPE_ITEMS: usize = 3;
pub const SHAPE_TEXT: usize = 200;
pub const SHAPE_DEPTH: usize = 8;

pub fn shape_of(value: &Value) -> String {
    let mut items = SHAPE_ITEMS;
    let mut text = SHAPE_TEXT;
    let mut depth = SHAPE_DEPTH;
    loop {
        let shaped = shortened(value, (items, text, depth)).to_string();
        if shaped.len() <= LARGEST_SHAPE || (items == 1 && text <= 20 && depth <= 1) {
            return shaped;
        }
        if items > 1 {
            items /= 2;
        } else if text > 20 {
            text /= 2;
        } else {
            depth -= 1;
        }
    }
}

fn shortened(value: &Value, (items, text, depth): (usize, usize, usize)) -> Value {
    match value {
        Value::String(content) if content.chars().count() > text => {
            Value::String(content.chars().take(text).collect())
        }
        Value::Array(list) if depth == 0 => Value::Array(Vec::new()),
        Value::Object(_) if depth == 0 => Value::Object(serde_json::Map::new()),
        Value::Array(list) => Value::Array(
            list.iter()
                .take(items)
                .map(|item| shortened(item, (items, text, depth - 1)))
                .collect(),
        ),
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, item)| (key.clone(), shortened(item, (items, text, depth - 1))))
                .collect(),
        ),
        other => other.clone(),
    }
}
