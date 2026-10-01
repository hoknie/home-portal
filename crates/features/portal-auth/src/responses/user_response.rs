use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct UserResponse {
    pub name: String,
    pub group: Option<String>,
    pub you: bool,
}
