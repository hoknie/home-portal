use super::{Condition, HttpStep, LoopMode, SetValue, Step};
use crate::types::{LogLevel, Operation, RunSettings};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepKind {
    If {
        condition: Condition,
        then: Vec<Step>,
        otherwise: Vec<Step>,
    },
    Loop {
        mode: LoopMode,
        max_iterations: u32,
        body: Vec<Step>,
    },
    Parallel {
        branches: Vec<Vec<Step>>,
    },
    Call {
        workflow: String,
        inputs: Vec<(String, String)>,
    },
    Stop {
        succeeded: bool,
        reason: Option<String>,
    },
    Set {
        variable: String,
        value: SetValue,
    },
    Wait {
        seconds: u64,
    },
    Transform {
        input: String,
        operations: Vec<Operation>,
    },
    Http(HttpStep),
    Script {
        run: RunSettings,
        env: Vec<(String, String)>,
        stdin: Option<String>,
    },
    Notify {
        channel: Option<String>,
        title: Option<String>,
        text: String,
    },
    Probe {
        service: String,
    },
    Nothing,
    Automation {
        automation: String,
        fields: Vec<(String, String)>,
        wait: bool,
    },
    Status {
        service: String,
    },
    Log {
        message: String,
        level: LogLevel,
    },
}

impl StepKind {
    pub fn name(&self) -> &'static str {
        match self {
            StepKind::If { .. } => "if",
            StepKind::Loop { .. } => "loop",
            StepKind::Parallel { .. } => "parallel",
            StepKind::Call { .. } => "workflow",
            StepKind::Stop { .. } => "stop",
            StepKind::Set { .. } => "set",
            StepKind::Wait { .. } => "wait",
            StepKind::Transform { .. } => "transform",
            StepKind::Http(_) => "http",
            StepKind::Script { .. } => "script",
            StepKind::Notify { .. } => "notify",
            StepKind::Probe { .. } => "probe",
            StepKind::Status { .. } => "status",
            StepKind::Nothing => "nothing",
            StepKind::Automation { .. } => "automation",
            StepKind::Log { .. } => "log",
        }
    }
}
