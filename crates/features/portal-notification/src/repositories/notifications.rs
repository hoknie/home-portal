use std::path::PathBuf;

use portal_config::Snapshot;
use toml_edit::{Array, DocumentMut, Item, Table, Value, value};

use crate::types::Rules;

pub const SECTION: &str = Rules::SECTION;
pub const LEGACY_KEYS: [&str; 2] = ["states", "recovered"];

pub fn section_mut(document: &mut DocumentMut) -> &mut Table {
    if !document.contains_key(SECTION) || document[SECTION].as_table().is_none() {
        let mut table = Table::new();
        table.set_implicit(true);
        document.insert(SECTION, Item::Table(table));
    }
    document[SECTION]
        .as_table_mut()
        .expect("the notifications table was just ensured")
}

pub fn channel_table<'a>(document: &'a mut DocumentMut, channel: &str) -> &'a mut Table {
    let section = section_mut(document);
    if section.get(channel).and_then(Item::as_table).is_none() {
        section.insert(channel, Item::Table(Table::new()));
    }
    section[channel]
        .as_table_mut()
        .expect("the channel table was just ensured")
}

pub fn write_rules(document: &mut DocumentMut, rules: &Rules) {
    let section = section_mut(document);
    section.set_implicit(false);
    let states: Array = rules.states.iter().map(String::as_str).collect();
    put(section, "states", Value::Array(states));
    put(section, "recovered", rules.recovered.into());
    if let Some(legacy) = section.get_mut(Rules::LEGACY).and_then(Item::as_table_mut) {
        for key in LEGACY_KEYS {
            legacy.remove(key);
        }
    }
}

pub fn rules_origin(snapshot: &Snapshot) -> Option<PathBuf> {
    snapshot.origins.table(SECTION).map(PathBuf::from)
}

pub fn channel_origin(snapshot: &Snapshot, channel: &str) -> Option<PathBuf> {
    snapshot
        .origins
        .table(&format!("{SECTION}.{channel}"))
        .or_else(|| snapshot.origins.table(SECTION))
        .map(PathBuf::from)
}

fn put(table: &mut Table, key: &str, wanted: Value) {
    let unchanged = table
        .get(key)
        .and_then(Item::as_value)
        .is_some_and(|existing| existing.to_string().trim() == wanted.to_string().trim());
    if !unchanged {
        let decor = table
            .get(key)
            .and_then(Item::as_value)
            .map(|existing| existing.decor().clone());
        let mut fresh = wanted;
        if let Some(decor) = decor {
            *fresh.decor_mut() = decor;
        }
        table[key] = value(fresh);
    }
}
