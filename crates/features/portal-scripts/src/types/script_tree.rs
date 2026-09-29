use super::ScriptFile;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScriptTree {
    pub folders: Vec<String>,
    pub files: Vec<ScriptFile>,
    pub left_out: usize,
}
