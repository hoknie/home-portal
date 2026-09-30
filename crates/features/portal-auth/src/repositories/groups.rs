use std::path::{Path, PathBuf};

use portal_config::Snapshot;
use portal_feature::Rights;
use toml_edit::{Array, ArrayOfTables, DocumentMut, InlineTable, Item, Table, Value, value};

use super::users::{GROUP, NAME, SECTION as USERS, remove_from};

pub const SECTION: &str = "groups";
pub const PERMISSIONS: &str = "permissions";

pub fn group_position(document: &DocumentMut, name: &str) -> Option<usize> {
    document
        .get(SECTION)?
        .as_array_of_tables()?
        .iter()
        .position(|table| table.get(NAME).and_then(Item::as_str) == Some(name))
}

pub fn permissions_of(rights: &Rights) -> Item {
    let mut table = InlineTable::new();
    for (area, actions) in rights.by_area() {
        let list: Array = actions.iter().map(|action| action.name()).collect();
        table.insert(area.name(), Value::Array(list));
    }
    Item::Value(Value::InlineTable(table))
}

pub fn append_group(document: &mut DocumentMut, name: &str, rights: &Rights) {
    let mut table = Table::new();
    table[NAME] = value(name);
    table[PERMISSIONS] = permissions_of(rights);
    if let Some(entries) = document
        .get_mut(SECTION)
        .and_then(Item::as_array_of_tables_mut)
    {
        entries.push(table);
        return;
    }
    let mut entries = ArrayOfTables::new();
    entries.push(table);
    document.insert(SECTION, Item::ArrayOfTables(entries));
}

pub fn set_group_entry(document: &mut DocumentMut, index: usize, name: &str, rights: &Rights) {
    let Some(table) = document
        .get_mut(SECTION)
        .and_then(Item::as_array_of_tables_mut)
        .and_then(|entries| entries.get_mut(index))
    else {
        return;
    };
    let mut fresh = Value::from(name);
    if let Some(existing) = table.get(NAME).and_then(Item::as_value) {
        *fresh.decor_mut() = existing.decor().clone();
    }
    table.insert(NAME, Item::Value(fresh));
    table.insert(PERMISSIONS, permissions_of(rights));
}

pub fn remove_group(document: &mut DocumentMut, index: usize) {
    remove_from(document, SECTION, index);
}

pub fn rename_members(document: &mut DocumentMut, old: &str, new: &str) {
    let Some(entries) = document
        .get_mut(USERS)
        .and_then(Item::as_array_of_tables_mut)
    else {
        return;
    };
    for table in entries.iter_mut() {
        if table.get(GROUP).and_then(Item::as_str) != Some(old) {
            continue;
        }
        let mut fresh = Value::from(new);
        if let Some(existing) = table.get(GROUP).and_then(Item::as_value) {
            *fresh.decor_mut() = existing.decor().clone();
        }
        table.insert(GROUP, Item::Value(fresh));
    }
}

pub fn group_origin(snapshot: &Snapshot, name: &str) -> Option<PathBuf> {
    let index = group_position(&snapshot.document, name)?;
    snapshot.origins.of(SECTION, index).map(Path::to_path_buf)
}

pub fn last_group_origin(snapshot: &Snapshot) -> Option<PathBuf> {
    let last = snapshot.origins.count(SECTION).checked_sub(1)?;
    snapshot.origins.of(SECTION, last).map(Path::to_path_buf)
}

pub fn member_origins(snapshot: &Snapshot, group: &str) -> Vec<PathBuf> {
    let Some(entries) = snapshot
        .document
        .get(USERS)
        .and_then(Item::as_array_of_tables)
    else {
        return Vec::new();
    };
    let mut found: Vec<PathBuf> = entries
        .iter()
        .enumerate()
        .filter(|(_, table)| table.get(GROUP).and_then(Item::as_str) == Some(group))
        .filter_map(|(index, _)| snapshot.origins.of(USERS, index).map(Path::to_path_buf))
        .collect();
    found.sort();
    found.dedup();
    found
}
