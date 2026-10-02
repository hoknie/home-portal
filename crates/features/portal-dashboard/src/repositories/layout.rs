use std::collections::HashMap;

use portal_widget::{SectionAppearance, SectionEntry, WidgetInstance, WidgetsSection};
use toml_edit::{DocumentMut, Table, value};

use super::library_moves::{
    SECTIONS_KEY, WIDGETS, ensure_dashboard, move_inline_into_library, put_tables, take_tables,
};
use super::table_sync::{APPEARANCE, SECTION_LOOK, appearance_item, sync, write_placement};
use crate::responses::WidgetView;
use crate::types::EditedLayout;

pub fn write_layout(document: &mut DocumentMut, edited: &EditedLayout) {
    move_inline_into_library(document);
    let existing = WidgetsSection::placements(document).unwrap_or_default();
    let existing_sections = existing_sections(document);
    let implicit = edited.only_the_implicit_section() && existing_sections.is_empty();
    let first_section = existing_sections
        .first()
        .map(|section| section.id.clone())
        .unwrap_or_else(|| SectionEntry::IMPLICIT.to_string());
    let Some(dashboard) = ensure_dashboard(document) else {
        return;
    };
    let mut widget_tables = take_tables(dashboard, WIDGETS);
    let mut section_tables = take_tables(dashboard, SECTIONS_KEY);
    let widget_positions = positions(&widget_tables);
    let section_positions = positions(&section_tables);
    let mut by_key: HashMap<String, (Table, WidgetInstance)> = widget_tables
        .drain(..)
        .zip(existing)
        .enumerate()
        .map(|(index, pair)| (WidgetView::key_of(index), pair))
        .collect();
    let mut widgets = Vec::new();
    for (index, widget) in edited.widgets.iter().enumerate() {
        let mut wanted = widget.instance.clone();
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
        if wanted.widget.is_none() {
            wanted.widget = old.widget.clone();
        }
        write_placement(&mut table, &old, &wanted);
        table.set_position(widget_positions.get(index).copied());
        widgets.push(table);
    }
    let mut sections_by_id: HashMap<String, (Table, SectionEntry)> = section_tables
        .drain(..)
        .zip(existing_sections)
        .map(|(table, section)| (section.id.clone(), (table, section)))
        .collect();
    let mut sections = Vec::new();
    if !implicit {
        for (index, section) in edited.sections.iter().enumerate() {
            let (mut table, old) = sections_by_id.remove(&section.id).unwrap_or_else(|| {
                (
                    Table::new(),
                    SectionEntry {
                        id: String::new(),
                        title: None,
                        appearance: SectionAppearance::default(),
                    },
                )
            });
            sync(&mut table, "id", &old.id, &section.id, || {
                Some(value(section.id.as_str()))
            });
            sync(&mut table, "title", &old.title, &section.title, || {
                section.title.as_deref().map(value)
            });
            sync(
                &mut table,
                APPEARANCE,
                &old.appearance,
                &section.appearance,
                || appearance_item(&section.appearance, &SECTION_LOOK),
            );
            table.set_position(section_positions.get(index).copied());
            sections.push(table);
        }
    }
    put_tables(dashboard, SECTIONS_KEY, sections);
    put_tables(dashboard, WIDGETS, widgets);
}

fn existing_sections(document: &DocumentMut) -> Vec<SectionEntry> {
    WidgetsSection::layout(document)
        .ok()
        .flatten()
        .filter(|layout| layout.explicit_sections)
        .map(|layout| layout.sections)
        .unwrap_or_default()
}

fn positions(tables: &[Table]) -> Vec<isize> {
    let mut positions: Vec<isize> = tables.iter().filter_map(Table::position).collect();
    positions.sort_unstable();
    positions
}
