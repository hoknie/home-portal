use super::Condition;
use crate::types::NumberSetting;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoopMode {
    Repeat(NumberSetting),
    ForEach(String),
    While(Condition),
}
