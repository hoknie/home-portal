use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct RawOutput {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputDeclaration {
    pub name: String,
    pub value: String,
    pub description: Option<String>,
}

impl OutputDeclaration {
    pub const MOST: usize = 32;
    pub const LONGEST_DESCRIPTION: usize = 200;
    pub const FIELD: &'static str = "outputs";
}
