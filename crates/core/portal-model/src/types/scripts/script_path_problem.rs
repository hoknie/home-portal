#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptPathProblem {
    Empty,
    Absolute,
    Climbs,
    Hidden,
    TooDeep,
}

impl ScriptPathProblem {
    pub fn message(self) -> &'static str {
        match self {
            ScriptPathProblem::Empty => "must name a script inside the scripts directory",
            ScriptPathProblem::Absolute => {
                "must be a path inside the scripts directory, not an absolute path"
            }
            ScriptPathProblem::Climbs => "must stay inside the scripts directory, without ..",
            ScriptPathProblem::Hidden => "must not be a hidden file or lie in a hidden folder",
            ScriptPathProblem::TooDeep => {
                "must lie directly in the scripts directory or in one of its subfolders, not deeper"
            }
        }
    }
}
