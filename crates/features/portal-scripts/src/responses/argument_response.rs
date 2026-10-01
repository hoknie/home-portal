use portal_model::ScriptArgument;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ArgumentResponse {
    pub name: String,
    pub option: bool,
    pub required: bool,
    #[serde(rename = "type")]
    pub kind: String,
    pub choices: Vec<String>,
    pub default: Option<String>,
    pub description: String,
}

impl ArgumentResponse {
    pub fn of(argument: &ScriptArgument) -> ArgumentResponse {
        ArgumentResponse {
            name: argument.name.clone(),
            option: argument.option,
            required: argument.required,
            kind: argument.kind.name().to_string(),
            choices: argument.kind.choices().to_vec(),
            default: argument.default.clone(),
            description: argument.description.clone(),
        }
    }
}
