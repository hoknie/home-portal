use portal_config::deserialize_section;
use serde::Deserialize;
use toml_edit::DocumentMut;

use super::{DashboardTable, Layout, SectionEntry, WidgetInstance};

#[derive(Debug, Default, Deserialize)]
pub struct WidgetsSection {
    #[serde(default)]
    pub dashboard: Option<DashboardTable>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Definition {
    pub field: String,
    pub settings_field: String,
    pub instance: WidgetInstance,
}

impl WidgetsSection {
    pub const PLACEMENTS: &'static str = "dashboard.widgets";
    pub const LIBRARY: &'static str = "dashboard.library";

    pub fn read(document: &DocumentMut) -> Result<Vec<WidgetInstance>, String> {
        Ok(Self::definitions(document)?
            .into_iter()
            .map(|definition| definition.instance)
            .collect())
    }

    pub fn placements(document: &DocumentMut) -> Result<Vec<WidgetInstance>, String> {
        Ok(Self::table(document)?
            .map(|table| table.widgets)
            .unwrap_or_default())
    }

    pub fn library(document: &DocumentMut) -> Result<Vec<WidgetInstance>, String> {
        Ok(Self::table(document)?
            .map(|table| table.library)
            .unwrap_or_default())
    }

    pub fn definitions(document: &DocumentMut) -> Result<Vec<Definition>, String> {
        let Some(table) = Self::table(document)? else {
            return Ok(Vec::new());
        };
        let named = |base: &str, index: usize, instance: &WidgetInstance| Definition {
            field: format!("{base}[{index}]"),
            settings_field: match &instance.id {
                Some(id) => format!("{base}.{id}.settings."),
                None => format!("{base}[{index}].settings."),
            },
            instance: instance.clone(),
        };
        let mut found: Vec<Definition> = table
            .library
            .iter()
            .enumerate()
            .map(|(index, instance)| named(Self::LIBRARY, index, instance))
            .collect();
        found.extend(
            table
                .widgets
                .iter()
                .enumerate()
                .filter(|(_, instance)| instance.is_inline())
                .map(|(index, instance)| named(Self::PLACEMENTS, index, instance)),
        );
        Ok(found)
    }

    pub fn derived_id(kind: &str, taken: &[String]) -> String {
        let base: String = kind
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() {
                    character.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect();
        let base = if base.starts_with(|character: char| character.is_ascii_lowercase()) {
            base
        } else {
            format!("widget-{base}")
        };
        if !taken.contains(&base) {
            return base;
        }
        (2..)
            .map(|number| format!("{base}-{number}"))
            .find(|candidate| !taken.contains(candidate))
            .unwrap_or(base)
    }

    pub fn with_ids(
        placements: &[WidgetInstance],
        library: &[WidgetInstance],
    ) -> Vec<WidgetInstance> {
        let mut taken: Vec<String> = library
            .iter()
            .chain(placements.iter().filter(|placement| placement.is_inline()))
            .filter_map(|entry| entry.id.clone())
            .collect();
        placements
            .iter()
            .map(|placement| {
                let mut named = placement.clone();
                if named.is_inline() && named.id.is_none() {
                    let id = Self::derived_id(&named.kind, &taken);
                    taken.push(id.clone());
                    named.id = Some(id);
                }
                named
            })
            .collect()
    }

    pub fn named_definitions(document: &DocumentMut) -> Result<Vec<WidgetInstance>, String> {
        let Some(table) = Self::table(document)? else {
            return Ok(Vec::new());
        };
        let mut found = table.library.clone();
        found.extend(
            Self::with_ids(&table.widgets, &table.library)
                .into_iter()
                .filter(WidgetInstance::is_inline),
        );
        Ok(found)
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
        let library = table.library;
        let widgets = Self::with_ids(&table.widgets, &library)
            .into_iter()
            .filter_map(|placement| {
                let mut resolved = match &placement.widget {
                    None => placement,
                    Some(name) => {
                        let definition = library
                            .iter()
                            .find(|entry| entry.id.as_deref() == Some(name.as_str()))?;
                        placement.placed_from(definition)
                    }
                };
                if resolved.section.is_none() {
                    resolved.section = Some(first.clone());
                }
                Some(resolved)
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
