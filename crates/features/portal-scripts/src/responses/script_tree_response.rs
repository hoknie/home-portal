use serde::Serialize;

use super::ScriptFileResponse;

#[derive(Debug, Clone, Serialize)]
pub struct ScriptTreeResponse {
    pub directory: String,
    pub exists: bool,
    pub user_id: u32,
    pub left_out: usize,
    pub inside: bool,
    pub folders: Vec<String>,
    pub files: Vec<ScriptFileResponse>,
}
