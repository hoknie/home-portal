use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum InputValue {
    Template(String),
    Literal(Value),
}

impl InputValue {
    pub fn template(&self) -> Option<&str> {
        match self {
            InputValue::Template(template) => Some(template),
            InputValue::Literal(_) => None,
        }
    }

    pub fn as_json(&self) -> Value {
        match self {
            InputValue::Template(template) => Value::String(template.clone()),
            InputValue::Literal(value) => value.clone(),
        }
    }
}
