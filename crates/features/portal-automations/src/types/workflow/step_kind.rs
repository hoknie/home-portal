use super::{Condition, HttpStep, LoopMode, SetValue, Step};
use crate::types::{LogLevel, NumberSetting, Operation, RunSettings};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepKind {
    If {
        condition: Condition,
        then: Vec<Step>,
        otherwise: Vec<Step>,
    },
    Loop {
        mode: LoopMode,
        max_iterations: NumberSetting,
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
        seconds: NumberSetting,
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
        timeout: NumberSetting,
        fail_on_error: bool,
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
    Break,
    Continue,
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
            StepKind::Break => "break",
            StepKind::Continue => "continue",
            StepKind::Automation { .. } => "automation",
            StepKind::Log { .. } => "log",
        }
    }
}
