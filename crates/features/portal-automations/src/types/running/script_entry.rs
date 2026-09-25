use super::Refusal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptEntry {
    pub path: String,
    pub problem: Option<Refusal>,
}
