use serde::Serialize;

use super::ScriptFileResponse;

#[derive(Debug, Clone, Serialize)]
pub struct ScriptTextResponse {
    pub path: String,
    pub content: String,
    pub revision: String,
    pub entry: ScriptFileResponse,
}
