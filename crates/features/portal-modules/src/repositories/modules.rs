use portal_feature::Module;
use toml_edit::{DocumentMut, Item, Table, Value};

pub const SECTION: &str = "modules";
pub const LEGACY_KEY: &str = "enabled";

pub fn write_switch(document: &mut DocumentMut, module: Module, on: bool) {
    if document
        .get(SECTION)
        .and_then(Item::as_table_like)
        .is_none()
    {
        document.insert(SECTION, Item::Table(Table::new()));
    }
    let Some(table) = document.get_mut(SECTION).and_then(Item::as_table_like_mut) else {
        return;
    };
    let mut fresh = Value::from(on);
    if let Some(existing) = table.get(module.name()).and_then(Item::as_value) {
        *fresh.decor_mut() = existing.decor().clone();
    }
    table.insert(module.name(), Item::Value(fresh));
}

pub fn remove_legacy_switch(document: &mut DocumentMut, module: Module) {
    let Some(section) = module.legacy_section() else {
        return;
    };
    if let Some(table) = document.get_mut(section).and_then(Item::as_table_like_mut) {
        table.remove(LEGACY_KEY);
    }
}
