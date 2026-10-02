use toml_edit::{Item, Key, Table};

pub fn renamed_in_place(table: &mut Table, old: &str, new: &str, item: Item) {
    let keys: Vec<Key> = table
        .iter()
        .filter_map(|(name, _)| table.key(name).cloned())
        .collect();
    let mut entries = Vec::with_capacity(keys.len());
    for key in keys {
        if let Some(existing) = table.remove(key.get()) {
            entries.push((key, existing));
        }
    }
    let mut item = Some(item);
    for (key, existing) in entries {
        if key.get() == old {
            let renamed = Key::new(new).with_leaf_decor(key.leaf_decor().clone());
            if let Some(item) = item.take() {
                table.insert_formatted(&renamed, item);
            }
        } else if key.get() != new {
            table.insert_formatted(&key, existing);
        }
    }
}
