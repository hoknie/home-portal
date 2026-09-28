use super::Ending;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Flow {
    Continue,
    End(Ending),
}
