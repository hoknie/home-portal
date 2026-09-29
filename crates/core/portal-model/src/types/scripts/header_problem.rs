#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderProblem {
    pub line: usize,
    pub message: String,
}
