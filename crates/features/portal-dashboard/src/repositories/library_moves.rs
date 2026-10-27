use portal_widget::WidgetInstance;
use toml_edit::{ArrayOfTables, DocumentMut, Item, Key, Table, value};

use crate::helpers::renamed_in_place;
use crate::types::{InlinePlacement, LibraryMoves, StoredWidgets};

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

fn stored_tables<'a>(document: &'a DocumentMut, key: &str) -> Vec<&'a Table> {
    document
        .get(DASHBOARD)
        .and_then(Item::as_table)
        .and_then(|dashboard| dashboard.get(key))
        .and_then(Item::as_array_of_tables)
        .map(|tables| tables.iter().collect())
        .unwrap_or_default()
}

pub fn stored_widgets(document: &DocumentMut) -> StoredWidgets {
    let inline = stored_tables(document, WIDGETS)
        .into_iter()
        .enumerate()
        .filter_map(|(index, table)| {
            text(table, "type").map(|kind| InlinePlacement {
                index,
                kind,
                id: text(table, "id"),
            })
        })
        .collect();
    StoredWidgets {
        dashboard: document.get(DASHBOARD).is_some(),
        library_ids: stored_tables(document, LIBRARY)
            .into_iter()
            .filter_map(|table| text(table, "id"))
            .collect(),
        inline,
    }
}

fn write_default_placements(document: &mut DocumentMut, kinds: &[String]) {
    let Some(dashboard) = ensure_dashboard(document) else {
        return;
    };
    let mut array = ArrayOfTables::new();
    for kind in kinds {
        let mut table = Table::new();
        table.insert("type", value(kind.as_str()));
        array.push(table);
    }
    dashboard.insert(WIDGETS, Item::ArrayOfTables(array));
}

pub fn apply_library_moves(document: &mut DocumentMut, moves: &LibraryMoves) {
    if !moves.defaults.is_empty() {
        write_default_placements(document, &moves.defaults);
    }
    let Some(dashboard) = ensure_dashboard(document) else {
        return;
    };
    let mut placements = take_tables(dashboard, WIDGETS);
    let mut library = take_tables(dashboard, LIBRARY);
    let last = library
        .iter()
        .chain(placements.iter())
        .filter_map(Table::position)
        .max()
        .unwrap_or(0);
    let mut moved = 0;
    for (index, id) in &moves.moves {
        let Some(placement) = placements.get_mut(*index) else {
            continue;
        };
        moved += 1;
        let mut definition = moved_definition(placement, id);
        definition.set_position(Some(last + moved));
        library.push(definition);
    }
    put_tables(dashboard, LIBRARY, library);
    put_tables(dashboard, WIDGETS, placements);
}

fn moved_definition(placement: &mut Table, id: &str) -> Table {
    let mut definition = Table::new();
    definition.insert("id", value(id));
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
    renamed_in_place(placement, "type", "widget", value(id));
    definition
}
