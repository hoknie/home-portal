use portal_widget::SectionEntry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionView {
    pub id: String,
    pub title: Option<String>,
}

impl SectionView {
    pub fn of(section: SectionEntry) -> SectionView {
        SectionView {
            id: section.id,
            title: section.title,
        }
    }
}
