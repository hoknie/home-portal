use super::StepKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub id: String,
    pub label: String,
    pub kind: StepKind,
}
