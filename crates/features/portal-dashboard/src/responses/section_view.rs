use portal_widget::{ResolvedSectionAppearance, SectionEntry};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SectionView {
    pub id: String,
    pub title: Option<String>,
    pub appearance: ResolvedSectionAppearance,
}

impl SectionView {
    pub fn of(section: SectionEntry) -> SectionView {
        SectionView {
            id: section.id,
            appearance: section.appearance.resolved(),
            title: section.title,
        }
    }
}
