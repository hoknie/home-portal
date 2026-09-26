use super::RecordKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    pub name: String,
    pub kind: RecordKind,
}
