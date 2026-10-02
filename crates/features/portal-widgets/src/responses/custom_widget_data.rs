use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::RenderedBlock;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CustomWidgetData {
    pub blocks: Vec<RenderedBlock>,
}

impl CustomWidgetData {
    pub fn without_buttons(self) -> CustomWidgetData {
        let blocks = self
            .blocks
            .into_iter()
            .filter_map(|block| {
                let mut value = serde_json::to_value(block).ok()?;
                if is_button(&value) {
                    return None;
                }
                strip(&mut value);
                serde_json::from_value(value).ok()
            })
            .collect();
        CustomWidgetData { blocks }
    }
}

fn is_button(value: &Value) -> bool {
    value.get("kind").and_then(Value::as_str) == Some("button")
}

fn strip(value: &mut Value) {
    if let Some(Value::Array(blocks)) = value.get_mut("blocks") {
        blocks.retain(|block| !is_button(block));
        blocks.iter_mut().for_each(strip);
    }
}
