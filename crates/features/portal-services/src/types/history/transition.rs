use portal_model::ServiceState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transition {
    pub at: i64,
    pub from: ServiceState,
    pub to: ServiceState,
    pub error: Option<String>,
}
