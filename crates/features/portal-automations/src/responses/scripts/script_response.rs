use serde::Serialize;

use super::{ArgumentResponse, HeaderProblemResponse};
use crate::types::ScriptEntry;

#[derive(Debug, Clone, Serialize)]
pub struct ScriptResponse {
    pub path: String,
    pub runnable: bool,
    pub problem: Option<String>,
    pub code: Option<String>,
    pub concerns: Option<String>,
    pub description: Option<String>,
    pub arguments: Vec<ArgumentResponse>,
    pub argument_problems: Vec<HeaderProblemResponse>,
}

impl ScriptResponse {
    pub fn of(entry: ScriptEntry) -> ScriptResponse {
        ScriptResponse {
            path: entry.path,
            runnable: entry.problem.is_none(),
            code: entry
                .problem
                .as_ref()
                .map(|refusal| refusal.code.name().to_string()),
            concerns: entry.problem.as_ref().map(|refusal| refusal.path.clone()),
            problem: entry.problem.map(|refusal| refusal.message),
            description: entry.header.description.clone(),
            arguments: entry
                .header
                .arguments
                .iter()
                .map(ArgumentResponse::of)
                .collect(),
            argument_problems: entry
                .header
                .problems
                .iter()
                .map(HeaderProblemResponse::of)
                .collect(),
        }
    }
}
