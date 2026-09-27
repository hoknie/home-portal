#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsersView {
    pub names: Vec<String>,
    pub you: String,
    pub editable: bool,
}
