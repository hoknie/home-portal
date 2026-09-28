use serde_json::Value;

use super::{Ending, Flow};

#[derive(Debug, Clone, PartialEq)]
pub struct StepReport {
    pub flow: Flow,
    pub result: Value,
    pub detail: String,
    pub output: Option<String>,
    pub shape: Option<String>,
}

impl StepReport {
    pub fn done(result: Value, detail: impl Into<String>) -> StepReport {
        StepReport {
            flow: Flow::Continue,
            result,
            detail: detail.into(),
            output: None,
            shape: None,
        }
    }

    pub fn failed(reason: impl Into<String>) -> StepReport {
        let reason = reason.into();
        StepReport {
            flow: Flow::End(Ending::Failed(reason.clone())),
            result: Value::Null,
            detail: reason,
            output: None,
            shape: None,
        }
    }

    pub fn ended(ending: Ending) -> StepReport {
        let detail = match &ending {
            Ending::Succeeded(reason) => reason.clone().unwrap_or_default(),
            Ending::Failed(reason) => reason.clone(),
            Ending::TimedOut => "the workflow's time ran out".to_string(),
            Ending::Stopped => "stopped".to_string(),
        };
        StepReport {
            flow: Flow::End(ending),
            result: Value::Null,
            detail,
            output: None,
            shape: None,
        }
    }

    pub fn with_flow(self, flow: Flow) -> StepReport {
        StepReport { flow, ..self }
    }
}
