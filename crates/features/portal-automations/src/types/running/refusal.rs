use std::path::Path;

use super::RefusalCode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    pub code: RefusalCode,
    pub path: String,
    pub message: String,
}

impl Refusal {
    pub fn new(code: RefusalCode, path: &Path, message: impl Into<String>) -> Refusal {
        Refusal {
            code,
            path: path.display().to_string(),
            message: message.into(),
        }
    }
}
