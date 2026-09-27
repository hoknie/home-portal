use portal_widget::{SectionEntry, WidgetInstance};

#[derive(Debug, Clone)]
pub struct LayoutView {
    pub sections: Vec<SectionEntry>,
    pub widgets: Vec<(usize, WidgetInstance)>,
}
