use toml_edit::{ArrayOfTables, DocumentMut, Item, Table};

use crate::types::Section;

pub fn wrapped(document: &DocumentMut) -> DocumentMut {
    let mut entry: Table = document.as_table().clone();
    entry.set_implicit(false);
    entry.set_position(Some(0));
    let mut entries = ArrayOfTables::new();
    entries.push(entry);
    let mut wrapped = DocumentMut::new();
    wrapped.insert(Section::Workflows.key(), Item::ArrayOfTables(entries));
    wrapped
}

pub fn empty_entries() -> DocumentMut {
    let mut document = DocumentMut::new();
    document.insert(
        Section::Workflows.key(),
        Item::ArrayOfTables(ArrayOfTables::new()),
    );
    document
}

pub fn unwrapped(document: &DocumentMut) -> DocumentMut {
    let entry = document
        .get(Section::Workflows.key())
        .and_then(Item::as_array_of_tables)
        .and_then(|entries| entries.get(0))
        .cloned()
        .unwrap_or_default();
    root_of(entry)
}

pub fn root_of(mut entry: Table) -> DocumentMut {
    let comment = entry
        .decor()
        .prefix()
        .and_then(|prefix| prefix.as_str())
        .map(|prefix| prefix.trim_start_matches(['\n', '\r']).to_string())
        .unwrap_or_default();
    entry.decor_mut().clear();
    entry.decor_mut().set_prefix(comment);
    entry.set_position(None);
    entry.set_implicit(false);
    DocumentMut::from(entry)
}

pub fn entry_id(document: &DocumentMut) -> Option<String> {
    document
        .get("id")
        .and_then(Item::as_str)
        .map(str::to_string)
}
