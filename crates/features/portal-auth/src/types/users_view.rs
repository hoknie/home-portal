#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsersView {
    pub members: Vec<(String, Option<String>)>,
    pub you: String,
    pub editable: bool,
}
