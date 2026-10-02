use portal_widget::{WidgetInstance, WidgetsSection};
use toml_edit::{DocumentMut, Table};

use super::library_moves::{
    LIBRARY, ensure_dashboard, move_inline_into_library, put_tables, take_tables,
};
use super::table_sync::write_definition;

pub fn write_library_entry(
    document: &mut DocumentMut,
    existing: Option<&str>,
    wanted: &WidgetInstance,
) -> String {
    move_inline_into_library(document);
    let stored = WidgetsSection::library(document).unwrap_or_default();
    let taken: Vec<String> = stored.iter().filter_map(|entry| entry.id.clone()).collect();
    let mut wanted = wanted.clone();
    let id = match (existing, &wanted.id) {
        (Some(id), _) => id.to_string(),
        (None, Some(id)) => id.clone(),
        (None, None) => WidgetsSection::derived_id(&wanted.kind, &taken),
    };
    wanted.id = Some(id.clone());
    let Some(dashboard) = ensure_dashboard(document) else {
        return id;
    };
    let mut tables = take_tables(dashboard, LIBRARY);
    let found = existing.and_then(|name| {
        stored
            .iter()
            .position(|entry| entry.id.as_deref() == Some(name))
    });
    match found {
        Some(index) => write_definition(&mut tables[index], &stored[index], &wanted),
        None => {
            let mut table = Table::new();
            write_definition(&mut table, &WidgetInstance::of(""), &wanted);
            let last = tables.iter().filter_map(Table::position).max();
            table.set_position(last.map(|position| position + 1));
            tables.push(table);
        }
    }
    put_tables(dashboard, LIBRARY, tables);
    id
}

pub fn remove_library_entry(document: &mut DocumentMut, id: &str) {
    move_inline_into_library(document);
    let stored = WidgetsSection::library(document).unwrap_or_default();
    let Some(dashboard) = ensure_dashboard(document) else {
        return;
    };
    let tables: Vec<Table> = take_tables(dashboard, LIBRARY)
        .into_iter()
        .zip(stored)
        .filter(|(_, entry)| entry.id.as_deref() != Some(id))
        .map(|(table, _)| table)
        .collect();
    put_tables(dashboard, LIBRARY, tables);
}
