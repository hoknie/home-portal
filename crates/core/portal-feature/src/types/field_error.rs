use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

impl FieldError {
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> FieldError {
        FieldError {
            field: field.into(),
            message: message.into(),
        }
    }

    pub fn prefixed(self, prefix: &str) -> FieldError {
        FieldError {
            field: format!("{prefix}{}", self.field),
            message: self.message,
        }
    }
}
