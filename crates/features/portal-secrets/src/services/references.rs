use std::collections::BTreeSet;

use toml_edit::{DocumentMut, Item, Table, Value};

pub const SECRET_KEY: &str = "secret";

pub fn referenced_secrets(document: &DocumentMut) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    walk_table(document.as_table(), &mut names);
    names
}

fn walk_table(table: &Table, names: &mut BTreeSet<String>) {
    for (key, item) in table.iter() {
        match item {
            Item::Value(Value::String(text)) if key == SECRET_KEY => {
                names.insert(text.value().to_string());
            }
            Item::Value(Value::InlineTable(inline)) => {
                for (inner, value) in inline.iter() {
                    if inner == SECRET_KEY
                        && let Some(text) = value.as_str()
                    {
                        names.insert(text.to_string());
                    }
                }
            }
            Item::Table(child) => walk_table(child, names),
            Item::ArrayOfTables(entries) => {
                for child in entries.iter() {
                    walk_table(child, names);
                }
            }
            _ => {}
        }
    }
}
