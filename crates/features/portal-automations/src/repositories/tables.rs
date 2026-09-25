use std::path::{Path, PathBuf};

use portal_config::Snapshot;
use toml_edit::{Array, ArrayOfTables, DocumentMut, InlineTable, Item, Table, Value, value};

pub fn position(document: &DocumentMut, section: &str, id: &str) -> Option<usize> {
    document
        .get(section)?
        .as_array_of_tables()?
        .iter()
        .position(|table| table.get("id").and_then(Item::as_str) == Some(id))
}

pub fn origin(snapshot: &Snapshot, section: &str, id: &str) -> Option<PathBuf> {
    let index = position(&snapshot.document, section, id)?;
    snapshot.origins.of(section, index).map(Path::to_path_buf)
}

pub fn push(document: &mut DocumentMut, section: &str, table: Table) {
    if let Some(entries) = document
        .get_mut(section)
        .and_then(Item::as_array_of_tables_mut)
    {
        entries.push(table);
        return;
    }
    let mut entries = ArrayOfTables::new();
    entries.push(table);
    document.insert(section, Item::ArrayOfTables(entries));
}

pub fn table_at<'a>(
    document: &'a mut DocumentMut,
    section: &str,
    index: usize,
) -> Option<&'a mut Table> {
    document
        .get_mut(section)
        .and_then(Item::as_array_of_tables_mut)
        .and_then(|entries| entries.get_mut(index))
}

pub fn remove_at(document: &mut DocumentMut, section: &str, index: usize) {
    let Some(entries) = document
        .get_mut(section)
        .and_then(Item::as_array_of_tables_mut)
    else {
        return;
    };
    let detached = entries
        .get(index)
        .and_then(|table| table.decor().prefix())
        .and_then(|prefix| prefix.as_str())
        .map(detached_comments)
        .unwrap_or_default();
    entries.remove(index);
    if detached.is_empty() {
        return;
    }
    if let Some(next) = entries.get_mut(index) {
        let existing = next
            .decor()
            .prefix()
            .and_then(|prefix| prefix.as_str())
            .unwrap_or_default()
            .trim_start_matches('\n')
            .to_string();
        next.decor_mut().set_prefix(format!("{detached}{existing}"));
        return;
    }
    let trailing = document.trailing().as_str().unwrap_or_default().to_string();
    document.set_trailing(format!("{trailing}{detached}"));
}

fn detached_comments(prefix: &str) -> String {
    prefix
        .rfind("\n\n")
        .map(|blank| prefix[..blank + 2].to_string())
        .unwrap_or_default()
}

pub fn set_nested(table: &mut Table, key: &str, wanted: InlineTable) {
    let Some(existing) = table.get_mut(key).and_then(Item::as_table_mut) else {
        set(table, key, Some(Value::InlineTable(wanted)));
        return;
    };
    let stale: Vec<String> = existing
        .iter()
        .map(|(name, _)| name.to_string())
        .filter(|name| !wanted.contains_key(name))
        .collect();
    for name in stale {
        existing.remove(&name);
    }
    for (name, value) in wanted.iter() {
        set(existing, name, Some(value.clone()));
    }
}

pub fn set(table: &mut Table, key: &str, wanted: Option<Value>) {
    let Some(wanted) = wanted else {
        table.remove(key);
        return;
    };
    let unchanged = table
        .get(key)
        .and_then(Item::as_value)
        .is_some_and(|existing| same(existing, &wanted));
    if !unchanged {
        table[key] = value(wanted);
    }
}

pub fn same(existing: &Value, wanted: &Value) -> bool {
    let parse = |value: &Value| {
        let mut bare = value.clone();
        bare.decor_mut().clear();
        format!("v = {bare}").parse::<toml::Table>().ok()
    };
    parse(existing).is_some_and(|existing| Some(existing) == parse(wanted))
}

pub fn tags_value(tags: &[String]) -> Option<Value> {
    (!tags.is_empty()).then(|| Value::Array(tags.iter().map(String::as_str).collect::<Array>()))
}
