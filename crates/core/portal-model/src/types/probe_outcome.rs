use super::{Diagnosis, ServiceState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeOutcome {
    pub state: ServiceState,
    pub latency_milliseconds: Option<u64>,
    pub error: Option<String>,
    pub diagnosis: Option<Diagnosis>,
}

impl ProbeOutcome {
    pub fn answered(state: ServiceState, latency_milliseconds: u64) -> ProbeOutcome {
        ProbeOutcome {
            state,
            latency_milliseconds: Some(latency_milliseconds),
            error: None,
            diagnosis: None,
        }
    }

    pub fn failed(
        state: ServiceState,
        latency_milliseconds: Option<u64>,
        error: String,
    ) -> ProbeOutcome {
        ProbeOutcome {
            state,
            latency_milliseconds,
            error: Some(error),
            diagnosis: Some(Diagnosis::Other),
        }
    }

    pub fn because(self, diagnosis: Diagnosis) -> ProbeOutcome {
        ProbeOutcome {
            diagnosis: Some(diagnosis),
            ..self
        }
    }
}
