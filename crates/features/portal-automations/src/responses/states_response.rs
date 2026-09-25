use serde::Serialize;

use crate::types::StateFilter;

#[derive(Debug, Clone, Serialize)]
pub struct StatesResponse {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub from: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub to: Vec<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub from_unknown: bool,
}

impl StatesResponse {
    pub fn of(states: &StateFilter) -> StatesResponse {
        StatesResponse {
            from: states.from.clone(),
            to: states.to.clone(),
            from_unknown: states.from_unknown,
        }
    }
}
