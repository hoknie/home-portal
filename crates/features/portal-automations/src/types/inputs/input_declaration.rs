use serde_json::Value;

use super::InputType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputDeclaration {
    pub name: String,
    pub input_type: InputType,
    pub default: Option<Value>,
    pub description: Option<String>,
}

impl InputDeclaration {
    pub fn text(name: &str) -> InputDeclaration {
        InputDeclaration {
            name: name.to_string(),
            input_type: InputType::Text,
            default: None,
            description: None,
        }
    }

    pub fn is_plain(&self) -> bool {
        self.input_type == InputType::Text && self.default.is_none() && self.description.is_none()
    }
}
