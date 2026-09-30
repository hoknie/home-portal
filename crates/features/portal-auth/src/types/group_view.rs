use portal_feature::Rights;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupView {
    pub name: String,
    pub builtin: bool,
    pub rights: Rights,
    pub members: Vec<String>,
}
