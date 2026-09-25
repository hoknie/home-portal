use portal_config::deserialize_section;
use serde::Deserialize;
use toml_edit::DocumentMut;

use super::{DashboardTable, Layout, SectionEntry, WidgetInstance};

#[derive(Debug, Default, Deserialize)]
pub struct WidgetsSection {
    #[serde(default)]
    pub dashboard: Option<DashboardTable>,
}

impl WidgetsSection {
    pub fn read(document: &DocumentMut) -> Result<Vec<WidgetInstance>, String> {
        Ok(Self::table(document)?
            .map(|table| table.widgets)
            .unwrap_or_default())
    }

    pub fn layout(document: &DocumentMut) -> Result<Option<Layout>, String> {
        let Some(table) = Self::table(document)? else {
            return Ok(None);
        };
        let explicit = !table.sections.is_empty();
        let sections = if explicit {
            table.sections
        } else {
            vec![SectionEntry::implicit()]
        };
        let first = sections
            .first()
            .map(|section| section.id.clone())
            .unwrap_or_else(|| SectionEntry::IMPLICIT.to_string());
        let widgets = table
            .widgets
            .into_iter()
            .map(|mut widget| {
                if widget.section.is_none() {
                    widget.section = Some(first.clone());
                }
                widget
            })
            .collect();
        Ok(Some(Layout {
            sections,
            widgets,
            explicit_sections: explicit,
        }))
    }

    fn table(document: &DocumentMut) -> Result<Option<DashboardTable>, String> {
        let section: WidgetsSection = deserialize_section(document)?;
        Ok(section.dashboard)
    }
}
