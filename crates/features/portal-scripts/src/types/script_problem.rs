use std::path::Path;

use super::ProblemCode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptProblem {
    pub code: ProblemCode,
    pub path: String,
    pub message: String,
}

impl ScriptProblem {
    pub fn new(code: ProblemCode, path: &Path, message: impl Into<String>) -> ScriptProblem {
        ScriptProblem {
            code,
            path: path.display().to_string(),
            message: message.into(),
        }
    }
}
