use portal_model::HeaderProblem;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct HeaderProblemResponse {
    pub line: usize,
    pub message: String,
}

impl HeaderProblemResponse {
    pub fn of(problem: &HeaderProblem) -> HeaderProblemResponse {
        HeaderProblemResponse {
            line: problem.line,
            message: problem.message.clone(),
        }
    }
}
