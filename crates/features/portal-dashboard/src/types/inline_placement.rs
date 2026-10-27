#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlinePlacement {
    pub index: usize,
    pub kind: String,
    pub id: Option<String>,
}
