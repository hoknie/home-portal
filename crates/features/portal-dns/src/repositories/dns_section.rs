use toml_edit::{Array, DocumentMut, Item, Table, TableLike, Value, value};

use crate::requests::DnsRequest;

pub const SECTION: &str = "dns";
pub const ADDRESSES: &str = "addresses";
pub const TLS: &str = "tls";
pub const HTTPS: &str = "https";

pub fn write_dns(document: &mut DocumentMut, request: &DnsRequest) {
    let table = section(document, SECTION);
    set(table, "enabled", request.enabled.into());
    set(table, "address", request.address.trim().into());
    set(table, "port", i64::from(request.port).into());
    set(table, "zones", strings(&request.zones).into());
    set(table, "ttl", i64::from(request.ttl).into());
    let addresses = child(table, ADDRESSES);
    let stale: Vec<String> = addresses
        .iter()
        .map(|(key, _)| key.to_string())
        .filter(|key| !request.addresses.contains_key(key))
        .collect();
    for key in stale {
        addresses.remove(&key);
    }
    for (environment, found) in &request.addresses {
        let written = match found.as_slice() {
            [one] => Value::from(one.trim()),
            many => strings(many).into(),
        };
        set(addresses, environment, written);
    }
    let tls = child(table, TLS);
    set(tls, "enabled", request.tls.enabled.into());
    optional(
        tls,
        "port",
        request.tls.port.map(|port| i64::from(port).into()),
    );
    optional(
        tls,
        "certificate",
        present(&request.tls.certificate).map(Value::from),
    );
    optional(tls, "key", present(&request.tls.key).map(Value::from));
    let https = child(table, HTTPS);
    set(https, "enabled", request.https.enabled.into());
    optional(https, "host", present(&request.https.host).map(Value::from));
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
