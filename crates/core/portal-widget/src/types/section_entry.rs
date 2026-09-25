use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionEntry {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
}

impl SectionEntry {
    pub const IMPLICIT: &'static str = "main";

    pub fn implicit() -> SectionEntry {
        SectionEntry {
            id: Self::IMPLICIT.to_string(),
            title: None,
        }
    }
}
