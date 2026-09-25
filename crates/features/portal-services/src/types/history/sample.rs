use portal_model::{Diagnosis, ServiceState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sample {
    pub at: i64,
    pub state: ServiceState,
    pub latency: Option<u32>,
    pub diagnosis: Option<Diagnosis>,
    pub covered: u32,
}

impl Sample {
    pub fn answered_seconds(&self) -> u64 {
        if self.state.answered() {
            u64::from(self.covered)
        } else {
            0
        }
    }
}
