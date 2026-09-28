use serde::Serialize;

use crate::types::Operator;

#[derive(Debug, Clone, Serialize)]
pub struct OperatorResponse {
    pub name: String,
    pub takes_right: bool,
}

impl OperatorResponse {
    pub fn of(operator: Operator) -> OperatorResponse {
        OperatorResponse {
            name: operator.name().to_string(),
            takes_right: operator.takes_right(),
        }
    }
}
