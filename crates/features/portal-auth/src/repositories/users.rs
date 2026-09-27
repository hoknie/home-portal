use std::path::{Path, PathBuf};

use portal_config::Snapshot;
use toml_edit::{ArrayOfTables, DocumentMut, Item, Table, Value, value};

pub const SECTION: &str = "users";
pub const NAME: &str = "name";
pub const PASSWORD_HASH: &str = "password_hash";

pub fn position(document: &DocumentMut, name: &str) -> Option<usize> {
    document
        .get(SECTION)?
        .as_array_of_tables()?
        .iter()
        .position(|table| table.get(NAME).and_then(Item::as_str) == Some(name))
}

pub fn append(document: &mut DocumentMut, name: &str, password_hash: &str) {
    let mut table = Table::new();
    table[NAME] = value(name);
    table[PASSWORD_HASH] = value(password_hash);
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

pub fn set_hash(document: &mut DocumentMut, index: usize, password_hash: &str) {
    let Some(table) = document
        .get_mut(SECTION)
        .and_then(Item::as_array_of_tables_mut)
        .and_then(|entries| entries.get_mut(index))
    else {
        return;
    };
    let mut fresh = Value::from(password_hash);
    if let Some(existing) = table.get(PASSWORD_HASH).and_then(Item::as_value) {
        *fresh.decor_mut() = existing.decor().clone();
    }
    table.insert(PASSWORD_HASH, Item::Value(fresh));
}

pub fn remove(document: &mut DocumentMut, index: usize) {
    let Some(entries) = document
        .get_mut(SECTION)
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

pub fn origin(snapshot: &Snapshot, name: &str) -> Option<PathBuf> {
    let index = position(&snapshot.document, name)?;
    snapshot.origins.of(SECTION, index).map(Path::to_path_buf)
}

pub fn last_origin(snapshot: &Snapshot) -> Option<PathBuf> {
    let count = snapshot.origins.count(SECTION);
    let last = count.checked_sub(1)?;
    snapshot.origins.of(SECTION, last).map(Path::to_path_buf)
}

fn detached_comments(prefix: &str) -> String {
    prefix
        .rfind("\n\n")
        .map(|blank| prefix[..blank + 2].to_string())
        .unwrap_or_default()
}
