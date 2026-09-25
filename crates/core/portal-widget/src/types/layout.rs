use super::{SectionEntry, WidgetInstance};

#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub sections: Vec<SectionEntry>,
    pub widgets: Vec<WidgetInstance>,
    pub explicit_sections: bool,
}
