#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusResult {
    pub state: String,
    pub since: Option<String>,
}
