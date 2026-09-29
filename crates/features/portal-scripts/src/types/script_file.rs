use std::time::SystemTime;

use portal_model::ScriptHeader;

use super::{ScriptProblem, Unreadable};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptFile {
    pub path: String,
    pub problem: Option<ScriptProblem>,
    pub header: ScriptHeader,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub mode: u32,
    pub unreadable: Option<Unreadable>,
    pub revision: Option<String>,
}

impl ScriptFile {
    pub const LARGEST_TEXT: u64 = 256 * 1024;
}
