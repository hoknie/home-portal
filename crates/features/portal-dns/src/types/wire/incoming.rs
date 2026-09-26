use super::Question;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Incoming {
    pub id: u16,
    pub recursion_desired: bool,
    pub query: bool,
    pub question: Option<Question>,
    pub question_type: u16,
}
