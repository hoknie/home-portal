use super::FilterCall;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placeholder<'a> {
    pub name: &'a str,
    pub filters: Result<Vec<FilterCall>, String>,
    pub inner_length: usize,
}
