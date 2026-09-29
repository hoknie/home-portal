use serde_json::Value;

use super::{Ending, Flow, Streams};

#[derive(Debug, Clone, PartialEq)]
pub struct StepReport {
    pub flow: Flow,
    pub result: Value,
    pub detail: String,
    pub output: Option<String>,
    pub shape: Option<String>,
    pub streams: Option<Streams>,
    pub log: Vec<String>,
}

impl StepReport {
    pub fn done(result: Value, detail: impl Into<String>) -> StepReport {
        StepReport {
            flow: Flow::Continue,
            result,
            detail: detail.into(),
            output: None,
            shape: None,
            streams: None,
            log: Vec::new(),
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
            streams: None,
            log: Vec::new(),
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
            streams: None,
            log: Vec::new(),
        }
    }

    pub fn with_flow(self, flow: Flow) -> StepReport {
        StepReport { flow, ..self }
    }

    pub fn logged(mut self, line: impl Into<String>) -> StepReport {
        self.log.push(line.into());
        self
    }

    pub fn with_log(mut self, mut lines: Vec<String>) -> StepReport {
        lines.append(&mut self.log);
        self.log = lines;
        self
    }
}
