use super::FilterCall;
use crate::types::Condition;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    Filter(FilterCall),
    Where(Condition),
    Map(String),
    SortBy { key: String, descending: bool },
    GroupBy(String),
    CountBy(String),
    Each(Vec<Operation>),
}

impl Operation {
    pub fn name(&self) -> &str {
        match self {
            Operation::Filter(call) => &call.name,
            Operation::Where(_) => "filter",
            Operation::Map(_) => "map",
            Operation::SortBy { .. } => "sort_by",
            Operation::GroupBy(_) => "group_by",
            Operation::CountBy(_) => "count_by",
            Operation::Each(_) => "each",
        }
    }
}
