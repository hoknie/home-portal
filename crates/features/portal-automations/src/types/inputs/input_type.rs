use serde_json::Value;

use crate::types::ValueType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputType {
    Text,
    Number,
    Boolean,
    List,
    Object,
}

impl InputType {
    pub const ALL: [InputType; 5] = [
        InputType::Text,
        InputType::Number,
        InputType::Boolean,
        InputType::List,
        InputType::Object,
    ];

    pub fn name(self) -> &'static str {
        match self {
            InputType::Text => "text",
            InputType::Number => "number",
            InputType::Boolean => "boolean",
            InputType::List => "list",
            InputType::Object => "object",
        }
    }

    pub fn named(name: &str) -> Option<InputType> {
        Self::ALL
            .into_iter()
            .find(|input_type| input_type.name() == name)
    }

    pub fn value_type(self) -> ValueType {
        match self {
            InputType::Text => ValueType::Text,
            InputType::Number => ValueType::Number,
            InputType::Boolean => ValueType::Boolean,
            InputType::List => ValueType::List,
            InputType::Object => ValueType::Object,
        }
    }

    pub fn fits(self, value: &Value) -> bool {
        ValueType::of(value) == self.value_type()
    }

    pub fn coerce(self, value: Value) -> Result<Value, String> {
        if value.is_null() || self.fits(&value) {
            return Ok(value);
        }
        let text = match &value {
            Value::String(text) => text.trim().to_string(),
            other => other.to_string(),
        };
        let read = match self {
            InputType::Text => Some(Value::String(match value {
                Value::String(text) => text,
                other => other.to_string(),
            })),
            InputType::Number => text
                .parse::<f64>()
                .ok()
                .filter(|number| number.is_finite())
                .and_then(serde_json::Number::from_f64)
                .map(|number| {
                    number
                        .as_f64()
                        .filter(|float| float.fract() == 0.0 && float.abs() < 9.0e15)
                        .map_or(Value::Number(number), |float| Value::from(float as i64))
                }),
            InputType::Boolean => match text.as_str() {
                "true" => Some(Value::Bool(true)),
                "false" => Some(Value::Bool(false)),
                _ => None,
            },
            InputType::List | InputType::Object => serde_json::from_str::<Value>(&text)
                .ok()
                .filter(|parsed| self.fits(parsed)),
        };
        read.ok_or_else(|| format!("must be {}, and \"{text}\" is not", self.described()))
    }

    pub fn described(self) -> &'static str {
        self.value_type().described()
    }
}
