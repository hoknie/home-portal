use schemars::JsonSchema;
use serde::Serialize;

use super::ArgumentResponse;
use crate::types::FilterDescription;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct FilterResponse {
    pub name: String,
    pub arguments: Vec<ArgumentResponse>,
    pub accepts: Vec<String>,
    pub gives: String,
    pub element: bool,
}

impl FilterResponse {
    pub fn of(filter: &FilterDescription) -> FilterResponse {
        FilterResponse {
            name: filter.name.to_string(),
            arguments: filter.arguments.iter().map(ArgumentResponse::of).collect(),
            accepts: filter
                .accepts
                .iter()
                .map(|value_type| value_type.name().to_string())
                .collect(),
            gives: filter.gives.name().to_string(),
            element: filter.element,
        }
    }
}
