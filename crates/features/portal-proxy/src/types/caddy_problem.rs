use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaddyProblem {
    Unreachable(String),
    Refused(String),
}

impl CaddyProblem {
    pub fn answered(&self) -> bool {
        matches!(self, CaddyProblem::Refused(_))
    }
}

impl fmt::Display for CaddyProblem {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CaddyProblem::Unreachable(message) | CaddyProblem::Refused(message) => {
                formatter.write_str(message)
            }
        }
    }
}
