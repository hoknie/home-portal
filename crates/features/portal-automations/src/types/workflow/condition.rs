use super::Operator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Condition {
    Compare {
        left: String,
        operator: Operator,
        right: Option<String>,
    },
    All(Vec<Condition>),
    Any(Vec<Condition>),
}
