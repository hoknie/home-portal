use serde::Serialize;

use crate::types::ScriptEntry;

#[derive(Debug, Clone, Serialize)]
pub struct ScriptResponse {
    pub path: String,
    pub runnable: bool,
    pub problem: Option<String>,
    pub code: Option<String>,
    pub concerns: Option<String>,
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
        }
    }
}
