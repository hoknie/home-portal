use portal_widget::{SectionEntry, WidgetInstance};
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
                })
                .collect(),
            widgets: self
                .widgets
                .into_iter()
                .map(|widget| EditedWidget {
                    key: widget.key,
                    instance: WidgetInstance {
                        kind: widget.kind.trim().to_string(),
                        id: blank_to_none(widget.id),
                        title: blank_to_none(widget.title),
                        settings: widget.settings,
                        environments: widget.environments,
                        public: widget.public,
                        section: blank_to_none(widget.section),
                        size: widget.size,
                    },
                })
                .collect(),
        }
    }
}
