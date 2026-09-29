use super::ArgumentKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptArgument {
    pub name: String,
    pub option: bool,
    pub required: bool,
    pub kind: ArgumentKind,
    pub default: Option<String>,
    pub description: String,
}
