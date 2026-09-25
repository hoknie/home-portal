#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StateFilter {
    pub from: Vec<String>,
    pub to: Vec<String>,
    pub from_unknown: bool,
}
