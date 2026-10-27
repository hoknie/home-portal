use portal_widget::{SectionEntry, WidgetInstance};

use super::PlannedEntry;

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutPlan {
    pub placements: Vec<PlannedEntry<WidgetInstance>>,
    pub sections: Vec<PlannedEntry<SectionEntry>>,
}
