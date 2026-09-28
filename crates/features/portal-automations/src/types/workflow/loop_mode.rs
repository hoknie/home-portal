use super::Condition;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoopMode {
    Repeat(u32),
    ForEach(String),
    While(Condition),
}
