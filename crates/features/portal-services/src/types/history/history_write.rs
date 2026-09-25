use super::HistoryLine;
use crate::services::ServiceHistory;

#[derive(Debug, Clone, PartialEq)]
pub enum HistoryWrite {
    Append(Vec<HistoryLine>),
    Rewrite(ServiceHistory),
}
