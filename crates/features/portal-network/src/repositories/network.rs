use toml_edit::{Array, DocumentMut, Item, Table, value};

use crate::types::NetworkSettings;

pub const SECTION: &str = "network";

pub fn write_network(document: &mut DocumentMut, settings: &NetworkSettings) {
    if !document.contains_table(SECTION) {
        document.insert(SECTION, Item::Table(Table::new()));
    }
    let table = &mut document[SECTION];
    table["address"] = value(settings.address.to_string());
    table["port"] = value(i64::from(settings.port));
    match &settings.public_url {
        Some(url) => table["public_url"] = value(url.as_str()),
        None => {
            if let Some(table) = table.as_table_mut() {
                table.remove("public_url");
            }
        }
    }
    let proxies: Array = settings
        .trusted_proxies
        .iter()
        .map(|network| network.to_string())
        .collect();
    table["trusted_proxies"] = value(proxies);
}
