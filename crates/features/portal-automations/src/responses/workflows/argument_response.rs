use schemars::JsonSchema;
use serde::Serialize;

use crate::types::{ArgumentDescription, ArgumentType};

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ArgumentResponse {
    pub name: String,
    #[serde(rename = "type")]
    pub argument_type: String,
    pub required: bool,
    pub choices: Vec<String>,
}

impl ArgumentResponse {
    pub fn of(argument: &ArgumentDescription) -> ArgumentResponse {
        let choices: &[&str] = if argument.argument_type == ArgumentType::Order {
            ArgumentType::ORDERS
        } else {
            &[]
        };
        ArgumentResponse {
            name: argument.name.to_string(),
            argument_type: argument.argument_type.name().to_string(),
            required: argument.required,
            choices: choices.iter().map(|choice| choice.to_string()).collect(),
        }
    }
}
