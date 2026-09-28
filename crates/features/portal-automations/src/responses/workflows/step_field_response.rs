use serde::Serialize;

use crate::types::FieldDescription;

#[derive(Debug, Clone, Serialize)]
pub struct StepFieldResponse {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub required: bool,
    pub default: Option<String>,
    pub minimum: Option<i64>,
    pub maximum: Option<i64>,
    pub choices: Vec<String>,
}

impl StepFieldResponse {
    pub fn of(field: &FieldDescription) -> StepFieldResponse {
        StepFieldResponse {
            name: field.name.to_string(),
            field_type: field.field_type.name().to_string(),
            required: field.required,
            default: field.default.map(str::to_string),
            minimum: field.minimum,
            maximum: field.maximum,
            choices: field
                .field_type
                .choices()
                .iter()
                .map(|choice| choice.to_string())
                .collect(),
        }
    }
}
