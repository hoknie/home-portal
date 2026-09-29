#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ScriptsSettings {
    pub editing: bool,
}

impl ScriptsSettings {
    pub const TABLE: &'static str = "scripts";
    pub const EDITING: &'static str = "editing";
}
