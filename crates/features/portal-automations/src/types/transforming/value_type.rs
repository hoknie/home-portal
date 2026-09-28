use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Text,
    Number,
    Boolean,
    List,
    Object,
    Null,
    Any,
}

impl ValueType {
    pub fn name(self) -> &'static str {
        match self {
            ValueType::Text => "text",
            ValueType::Number => "number",
            ValueType::Boolean => "boolean",
            ValueType::List => "list",
            ValueType::Object => "object",
            ValueType::Null => "null",
            ValueType::Any => "any",
        }
    }

    pub fn described(self) -> &'static str {
        match self {
            ValueType::Text => "text",
            ValueType::Number => "a number",
            ValueType::Boolean => "a boolean",
            ValueType::List => "a list",
            ValueType::Object => "an object",
            ValueType::Null => "nothing",
            ValueType::Any => "any value",
        }
    }

    pub fn of(value: &Value) -> ValueType {
        match value {
            Value::Null => ValueType::Null,
            Value::Bool(_) => ValueType::Boolean,
            Value::Number(_) => ValueType::Number,
            Value::String(_) => ValueType::Text,
            Value::Array(_) => ValueType::List,
            Value::Object(_) => ValueType::Object,
        }
    }
}
