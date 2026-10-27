use portal_widget::{SectionEntry, WidgetInstance, WidgetsSection};
use toml_edit::{DocumentMut, Table, value};

use super::library_moves::{SECTIONS_KEY, WIDGETS, ensure_dashboard, put_tables, take_tables};
use super::table_sync::{APPEARANCE, SECTION_LOOK, appearance_item, sync, write_placement};
use crate::types::LayoutPlan;

pub fn stored_layout(document: &DocumentMut) -> (Vec<WidgetInstance>, Vec<SectionEntry>) {
    let placements = WidgetsSection::placements(document).unwrap_or_default();
    let sections = WidgetsSection::layout(document)
        .ok()
        .flatten()
        .filter(|layout| layout.explicit_sections)
        .map(|layout| layout.sections)
        .unwrap_or_default();
    (placements, sections)
}

pub fn apply_layout(document: &mut DocumentMut, plan: &LayoutPlan) {
    let Some(dashboard) = ensure_dashboard(document) else {
        return;
    };
    let widget_tables = take_tables(dashboard, WIDGETS);
    let section_tables = take_tables(dashboard, SECTIONS_KEY);
    let widget_positions = positions(&widget_tables);
    let section_positions = positions(&section_tables);
    let mut widget_tables: Vec<Option<Table>> = widget_tables.into_iter().map(Some).collect();
    let mut section_tables: Vec<Option<Table>> = section_tables.into_iter().map(Some).collect();
    let mut widgets = Vec::new();
    for (index, entry) in plan.placements.iter().enumerate() {
        let mut table = taken(&mut widget_tables, entry.stored);
        write_placement(&mut table, &entry.old, &entry.wanted);
        table.set_position(widget_positions.get(index).copied());
        widgets.push(table);
    }
    let mut sections = Vec::new();
    for (index, entry) in plan.sections.iter().enumerate() {
        let mut table = taken(&mut section_tables, entry.stored);
        let (old, section) = (&entry.old, &entry.wanted);
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
    put_tables(dashboard, SECTIONS_KEY, sections);
    put_tables(dashboard, WIDGETS, widgets);
}

fn taken(tables: &mut [Option<Table>], stored: Option<usize>) -> Table {
    stored
        .and_then(|index| tables.get_mut(index))
        .and_then(Option::take)
        .unwrap_or_default()
}

fn positions(tables: &[Table]) -> Vec<isize> {
    let mut positions: Vec<isize> = tables.iter().filter_map(Table::position).collect();
    positions.sort_unstable();
    positions
}
