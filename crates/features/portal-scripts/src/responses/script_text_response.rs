use schemars::JsonSchema;
use serde::Serialize;

use super::ScriptFileResponse;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ScriptTextResponse {
    pub path: String,
    pub content: String,
    pub revision: String,
    pub entry: ScriptFileResponse,
}
