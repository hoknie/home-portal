use serde::Deserialize;

use crate::types::Rules;

#[derive(Debug, Clone, Deserialize)]
pub struct RulesRequest {
    #[serde(default = "Rules::default_states")]
    pub states: Vec<String>,
    #[serde(default = "RulesRequest::yes")]
    pub recovered: bool,
}

impl RulesRequest {
    pub fn yes() -> bool {
        true
    }

    pub fn into_rules(self) -> Rules {
        Rules {
            states: self.states,
            recovered: self.recovered,
        }
    }
}
