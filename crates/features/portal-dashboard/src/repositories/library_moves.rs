use portal_widget::{WidgetInstance, WidgetsSection};
use toml_edit::{ArrayOfTables, DocumentMut, Item, Key, Table, value};

use crate::helpers::renamed_in_place;
use crate::services::DEFAULT_WIDGETS;

pub const DASHBOARD: &str = "dashboard";
pub const WIDGETS: &str = "widgets";
pub const LIBRARY: &str = "library";
pub const SECTIONS_KEY: &str = "sections";

pub fn ensure_dashboard(document: &mut DocumentMut) -> Option<&mut Table> {
    if !document.get(DASHBOARD).is_some_and(Item::is_table) {
        let mut table = Table::new();
        table.set_implicit(true);
        document.insert(DASHBOARD, Item::Table(table));
    }
    document.get_mut(DASHBOARD).and_then(Item::as_table_mut)
}

pub fn take_tables(dashboard: &mut Table, key: &str) -> Vec<Table> {
    match dashboard.remove(key) {
        Some(Item::ArrayOfTables(tables)) => tables.into_iter().collect(),
        _ => Vec::new(),
    }
}

pub fn put_tables(dashboard: &mut Table, key: &str, tables: Vec<Table>) {
    if tables.is_empty() {
        return;
    }
    let mut array = ArrayOfTables::new();
    for table in tables {
        array.push(table);
    }
    dashboard.insert(key, Item::ArrayOfTables(array));
}

fn text(table: &Table, key: &str) -> Option<String> {
    table.get(key).and_then(Item::as_str).map(String::from)
}

fn write_the_default_layout(document: &mut DocumentMut) {
    if document.get(DASHBOARD).is_some() {
        return;
    }
    let Some(dashboard) = ensure_dashboard(document) else {
        return;
    };
    let mut array = ArrayOfTables::new();
    for kind in DEFAULT_WIDGETS {
        let mut table = Table::new();
        table.insert("type", value(kind));
        array.push(table);
    }
    dashboard.insert(WIDGETS, Item::ArrayOfTables(array));
}

pub fn move_inline_into_library(document: &mut DocumentMut) {
    write_the_default_layout(document);
    let Some(dashboard) = ensure_dashboard(document) else {
        return;
    };
    let mut placements = take_tables(dashboard, WIDGETS);
    let mut library = take_tables(dashboard, LIBRARY);
    let mut taken: Vec<String> = library
        .iter()
        .chain(placements.iter().filter(|table| table.contains_key("type")))
        .filter_map(|table| text(table, "id"))
        .collect();
    let last = library
        .iter()
        .chain(placements.iter())
        .filter_map(Table::position)
        .max()
        .unwrap_or(0);
    let mut moved = 0;
    for placement in &mut placements {
        let Some(kind) = text(placement, "type") else {
            continue;
        };
        let id = match text(placement, "id") {
            Some(id) => id,
            None => {
                let id = WidgetsSection::derived_id(&kind, &taken);
                taken.push(id.clone());
                id
            }
        };
        let mut definition = Table::new();
        definition.insert("id", value(id.as_str()));
        let order: Vec<String> = placement.iter().map(|(key, _)| key.to_string()).collect();
        for key in order {
            if !WidgetInstance::DEFINITION_KEYS.contains(&key.as_str()) || key == "id" {
                continue;
            }
            if key == "type" {
                if let Some(item) = placement.get(&key) {
                    definition.insert(&key, item.clone());
                }
                continue;
            }
            if let Some((name, item)) = placement.remove_entry(&key) {
                definition.insert_formatted(&Key::new(name.get()), item);
            }
        }
        placement.remove("id");
        renamed_in_place(placement, "type", "widget", value(id.as_str()));
        moved += 1;
        definition.set_position(Some(last + moved));
        library.push(definition);
    }
    put_tables(dashboard, LIBRARY, library);
    put_tables(dashboard, WIDGETS, placements);
}
