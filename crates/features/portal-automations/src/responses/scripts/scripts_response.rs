use serde::Serialize;

use super::ScriptResponse;

#[derive(Debug, Clone, Serialize)]
pub struct ScriptsResponse {
    pub directory: String,
    pub exists: bool,
    pub user_id: u32,
    pub editing: bool,
    pub scripts: Vec<ScriptResponse>,
}
