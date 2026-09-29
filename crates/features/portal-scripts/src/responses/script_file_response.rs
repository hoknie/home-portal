use serde::Serialize;
use time::OffsetDateTime;

use super::{ArgumentResponse, HeaderProblemResponse};
use crate::types::ScriptFile;

#[derive(Debug, Clone, Serialize)]
pub struct ScriptFileResponse {
    pub path: String,
    pub folder: Option<String>,
    pub name: String,
    pub size: u64,
    #[serde(with = "time::serde::rfc3339::option")]
    pub modified: Option<OffsetDateTime>,
    pub mode: String,
    pub runnable: bool,
    pub problem: Option<String>,
    pub code: Option<String>,
    pub concerns: Option<String>,
    pub text: bool,
    pub unreadable: Option<String>,
    pub revision: Option<String>,
    pub description: Option<String>,
    pub arguments: Vec<ArgumentResponse>,
    pub argument_problems: Vec<HeaderProblemResponse>,
}

impl ScriptFileResponse {
    pub fn of(file: &ScriptFile) -> ScriptFileResponse {
        let (folder, name) = match file.path.split_once('/') {
            Some((folder, name)) => (Some(folder.to_string()), name.to_string()),
            None => (None, file.path.clone()),
        };
        ScriptFileResponse {
            path: file.path.clone(),
            folder,
            name,
            size: file.size,
            modified: file.modified.map(OffsetDateTime::from),
            mode: format!("{:04o}", file.mode),
            runnable: file.problem.is_none(),
            problem: file.problem.as_ref().map(|problem| problem.message.clone()),
            code: file
                .problem
                .as_ref()
                .map(|problem| problem.code.name().to_string()),
            concerns: file.problem.as_ref().map(|problem| problem.path.clone()),
            text: file.unreadable.is_none(),
            unreadable: file.unreadable.map(|reason| reason.name().to_string()),
            revision: file.revision.clone(),
            description: file.header.description.clone(),
            arguments: file
                .header
                .arguments
                .iter()
                .map(ArgumentResponse::of)
                .collect(),
            argument_problems: file
                .header
                .problems
                .iter()
                .map(HeaderProblemResponse::of)
                .collect(),
        }
    }
}
