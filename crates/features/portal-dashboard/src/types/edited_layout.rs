use portal_widget::SectionEntry;

use super::EditedWidget;

#[derive(Debug, Clone, PartialEq)]
pub struct EditedLayout {
    pub sections: Vec<SectionEntry>,
    pub widgets: Vec<EditedWidget>,
}

impl EditedLayout {
    pub fn only_the_implicit_section(&self) -> bool {
        matches!(
            self.sections.as_slice(),
            [only] if only.id == SectionEntry::IMPLICIT && only.title.is_none()
        )
    }
}
