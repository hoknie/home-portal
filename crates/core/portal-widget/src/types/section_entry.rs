use serde::{Deserialize, Serialize};

use super::SectionAppearance;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionEntry {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub appearance: SectionAppearance,
}

impl SectionEntry {
    pub const IMPLICIT: &'static str = "main";

    pub fn implicit() -> SectionEntry {
        SectionEntry {
            id: Self::IMPLICIT.to_string(),
            title: None,
            appearance: SectionAppearance::default(),
        }
    }
}
