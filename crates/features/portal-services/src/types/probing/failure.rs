use portal_model::{Diagnosis, ProbeOutcome, ServiceState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    pub state: ServiceState,
    pub diagnosis: Diagnosis,
    pub message: String,
}

impl Failure {
    pub fn new(state: ServiceState, diagnosis: Diagnosis, message: impl Into<String>) -> Failure {
        Failure {
            state,
            diagnosis,
            message: message.into(),
        }
    }

    pub fn into_outcome(self, latency_milliseconds: Option<u64>) -> ProbeOutcome {
        ProbeOutcome::failed(self.state, latency_milliseconds, self.message).because(self.diagnosis)
    }
}
