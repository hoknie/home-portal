use toml_edit::{Array, DocumentMut, Item, Table, TableLike, Value, value};

use crate::types::DnsChoice;

pub const SECTION: &str = "dns";
pub const ADDRESSES: &str = "addresses";
pub const TLS: &str = "tls";
pub const HTTPS: &str = "https";

pub fn write_dns(document: &mut DocumentMut, choice: &DnsChoice) {
    let table = section(document, SECTION);
    set(table, "enabled", choice.enabled.into());
    set(table, "address", choice.address.trim().into());
    set(table, "port", i64::from(choice.port).into());
    set(table, "zones", strings(&choice.zones).into());
    set(table, "ttl", i64::from(choice.ttl).into());
    let addresses = child(table, ADDRESSES);
    let stale: Vec<String> = addresses
        .iter()
        .map(|(key, _)| key.to_string())
        .filter(|key| !choice.addresses.contains_key(key))
        .collect();
    for key in stale {
        addresses.remove(&key);
    }
    for (environment, found) in &choice.addresses {
        let written = match found.as_slice() {
            [one] => Value::from(one.trim()),
            many => strings(many).into(),
        };
        set(addresses, environment, written);
    }
    let tls = child(table, TLS);
    set(tls, "enabled", choice.tls.enabled.into());
    optional(
        tls,
        "port",
        choice.tls.port.map(|port| i64::from(port).into()),
    );
    optional(
        tls,
        "certificate",
        present(&choice.tls.certificate).map(Value::from),
    );
    optional(tls, "key", present(&choice.tls.key).map(Value::from));
    let https = child(table, HTTPS);
    set(https, "enabled", choice.https.enabled.into());
    optional(https, "host", present(&choice.https.host).map(Value::from));
}

fn present(text: &Option<String>) -> Option<&str> {
    text.as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty())
}

fn strings(values: &[String]) -> Array {
    values.iter().map(|text| text.trim()).collect()
}

fn section<'a>(document: &'a mut DocumentMut, name: &str) -> &'a mut dyn TableLike {
    if document.get(name).and_then(Item::as_table_like).is_none() {
        document[name] = Item::Table(Table::new());
    }
    document[name]
        .as_table_like_mut()
        .expect("the section was just made a table")
}

fn child<'a>(table: &'a mut dyn TableLike, name: &str) -> &'a mut dyn TableLike {
    if table.get(name).and_then(Item::as_table_like).is_none() {
        let mut fresh = Table::new();
        fresh.set_implicit(false);
        table.insert(name, Item::Table(fresh));
    }
    table
        .get_mut(name)
        .and_then(Item::as_table_like_mut)
        .expect("the table was just made")
}

fn set(table: &mut dyn TableLike, key: &str, mut fresh: Value) {
    if let Some(existing) = table.get(key).and_then(Item::as_value) {
        *fresh.decor_mut() = existing.decor().clone();
    }
    table.insert(key, value(fresh));
}

fn optional(table: &mut dyn TableLike, key: &str, fresh: Option<Value>) {
    match fresh {
        Some(fresh) => set(table, key, fresh),
        None => {
            table.remove(key);
        }
    }
}
