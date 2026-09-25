use std::collections::HashMap;

use portal_widget::{SectionEntry, WidgetInstance, WidgetSize, WidgetsSection};
use toml_edit::{Array, ArrayOfTables, DocumentMut, Item, Table, value};

use crate::helpers::{toml_of, unique_id};
use crate::responses::WidgetView;
use crate::types::EditedLayout;

pub const DASHBOARD: &str = "dashboard";
pub const WIDGETS: &str = "widgets";
pub const SECTIONS: &str = "sections";

pub fn write_layout(document: &mut DocumentMut, edited: &EditedLayout) {
    let existing_widgets = WidgetsSection::read(document).unwrap_or_default();
    let existing_sections = existing_sections(document);
    let implicit = edited.only_the_implicit_section() && existing_sections.is_empty();
    let first_section = existing_sections
        .first()
        .map(|section| section.id.clone())
        .unwrap_or_else(|| SectionEntry::IMPLICIT.to_string());
    ensure_dashboard(document);
    let Some(dashboard) = document.get_mut(DASHBOARD).and_then(Item::as_table_mut) else {
        return;
    };
    let mut widget_tables = take_tables(dashboard, WIDGETS);
    let mut section_tables = take_tables(dashboard, SECTIONS);
    let widget_positions = positions(&widget_tables);
    let section_positions = positions(&section_tables);
    let mut by_key: HashMap<String, (Table, WidgetInstance)> = widget_tables
        .drain(..)
        .zip(existing_widgets)
        .enumerate()
        .map(|(index, (table, instance))| (WidgetView::key_of(index, &instance), (table, instance)))
        .collect();
    let mut taken: Vec<String> = edited
        .widgets
        .iter()
        .filter_map(|widget| widget.instance.id.clone())
        .collect();
    let mut widgets = ArrayOfTables::new();
    for (index, widget) in edited.widgets.iter().enumerate() {
        let mut wanted = widget.instance.clone();
        if wanted.id.is_none() {
            let id = unique_id(&wanted.kind, &taken);
            taken.push(id.clone());
            wanted.id = Some(id);
        }
        if implicit {
            wanted.section = None;
        }
        let (mut table, mut old) = widget
            .key
            .as_ref()
            .and_then(|key| by_key.remove(key))
            .unwrap_or_else(|| (Table::new(), WidgetInstance::of("")));
        if old.section.is_none() && !implicit {
            old.section = Some(first_section.clone());
        }
        write_widget(&mut table, &old, &wanted);
        table.set_position(widget_positions.get(index).copied());
        widgets.push(table);
    }
    let mut sections_by_id: HashMap<String, (Table, SectionEntry)> = section_tables
        .drain(..)
        .zip(existing_sections)
        .map(|(table, section)| (section.id.clone(), (table, section)))
        .collect();
    let mut sections = ArrayOfTables::new();
    if !implicit {
        for (index, section) in edited.sections.iter().enumerate() {
            let (mut table, old) = sections_by_id.remove(&section.id).unwrap_or_else(|| {
                (
                    Table::new(),
                    SectionEntry {
                        id: String::new(),
                        title: None,
                    },
                )
            });
            sync(&mut table, "id", &old.id, &section.id, || {
                Some(value(section.id.as_str()))
            });
            sync(&mut table, "title", &old.title, &section.title, || {
                section.title.as_deref().map(value)
            });
            table.set_position(section_positions.get(index).copied());
            sections.push(table);
        }
    }
    if !sections.is_empty() {
        dashboard.insert(SECTIONS, Item::ArrayOfTables(sections));
    }
    if !widgets.is_empty() {
        dashboard.insert(WIDGETS, Item::ArrayOfTables(widgets));
    }
}

fn write_widget(table: &mut Table, old: &WidgetInstance, wanted: &WidgetInstance) {
    sync(table, "type", &old.kind, &wanted.kind, || {
        Some(value(wanted.kind.as_str()))
    });
    sync(table, "id", &old.id, &wanted.id, || {
        wanted.id.as_deref().map(value)
    });
    sync(table, "title", &old.title, &wanted.title, || {
        wanted.title.as_deref().map(value)
    });
    sync(table, "section", &old.section, &wanted.section, || {
        wanted.section.as_deref().map(value)
    });
    sync(table, "size", &old.size, &wanted.size, || {
        (wanted.size != WidgetSize::Full).then(|| value(wanted.size.name()))
    });
    sync(
        table,
        "environments",
        &old.environments,
        &wanted.environments,
        || {
            wanted.environments.as_ref().map(|names| {
                let list: Array = names.iter().map(String::as_str).collect();
                value(list)
            })
        },
    );
    sync(table, "public", &old.public, &wanted.public, || {
        wanted.public.then(|| value(true))
    });
    let empty = wanted
        .settings
        .as_object()
        .is_none_or(serde_json::Map::is_empty);
    sync(table, "settings", &old.settings, &wanted.settings, || {
        if empty {
            None
        } else {
            toml_of(&wanted.settings).map(Item::Value)
        }
    });
}

fn sync<T: PartialEq>(
    table: &mut Table,
    key: &str,
    old: &T,
    wanted: &T,
    item: impl FnOnce() -> Option<Item>,
) {
    if old == wanted {
        return;
    }
    match item() {
        Some(item) => {
            table.insert(key, item);
        }
        None => {
            table.remove(key);
        }
    }
}

fn existing_sections(document: &DocumentMut) -> Vec<SectionEntry> {
    WidgetsSection::layout(document)
        .ok()
        .flatten()
        .filter(|layout| layout.explicit_sections)
        .map(|layout| layout.sections)
        .unwrap_or_default()
}

fn ensure_dashboard(document: &mut DocumentMut) {
    if document.get(DASHBOARD).is_some_and(Item::is_table) {
        return;
    }
    let mut table = Table::new();
    table.set_implicit(true);
    document.insert(DASHBOARD, Item::Table(table));
}

fn take_tables(dashboard: &mut Table, key: &str) -> Vec<Table> {
    match dashboard.remove(key) {
        Some(Item::ArrayOfTables(tables)) => tables.into_iter().collect(),
        _ => Vec::new(),
    }
}

fn positions(tables: &[Table]) -> Vec<isize> {
    let mut positions: Vec<isize> = tables.iter().filter_map(Table::position).collect();
    positions.sort_unstable();
    positions
}
