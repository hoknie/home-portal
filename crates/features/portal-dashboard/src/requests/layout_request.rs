use portal_widget::{SectionAppearance, SectionEntry, WidgetInstance};
use serde::Deserialize;

use super::{SectionRequest, WidgetRequest};
use crate::types::{EditedLayout, EditedWidget};

#[derive(Debug, Clone, Deserialize)]
pub struct LayoutRequest {
    #[serde(default)]
    pub sections: Vec<SectionRequest>,
    #[serde(default)]
    pub widgets: Vec<WidgetRequest>,
}

impl LayoutRequest {
    pub fn into_layout(self) -> EditedLayout {
        let blank_to_none = |text: Option<String>| {
            text.map(|text| text.trim().to_string())
                .filter(|text| !text.is_empty())
        };
        EditedLayout {
            sections: self
                .sections
                .into_iter()
                .map(|section| SectionEntry {
                    id: section.id.trim().to_string(),
                    title: blank_to_none(section.title),
                    appearance: without_defaults_of_section(section.appearance),
                })
                .collect(),
            widgets: self
                .widgets
                .into_iter()
                .map(|widget| EditedWidget {
                    key: widget.key,
                    instance: WidgetInstance {
                        widget: blank_to_none(widget.widget),
                        section: blank_to_none(widget.section),
                        column: widget.column,
                        row: widget.row,
                        width: Some(widget.width),
                        height: widget.height,
                        ..WidgetInstance::of("")
                    },
                })
                .collect(),
        }
    }
}

fn without_defaults_of_section(appearance: SectionAppearance) -> SectionAppearance {
    if !appearance.problems().is_empty() {
        return appearance;
    }
    SectionAppearance::of(appearance.resolved())
}
