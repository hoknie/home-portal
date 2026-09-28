use toml_edit::{Item, Table, Value, value};

pub fn put(table: &mut Table, key: &str, wanted: Option<Value>) {
    let Some(wanted) = wanted else {
        table.remove(key);
        return;
    };
    let unchanged = table
        .get(key)
        .and_then(Item::as_value)
        .is_some_and(|existing| existing.to_string().trim() == wanted.to_string().trim());
    if !unchanged {
        table[key] = value(wanted);
    }
}
