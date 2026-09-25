use portal_model::{Publication, TlsPolicy};
use toml_edit::{Array, DocumentMut, InlineTable, Item, Table, TableLike, Value, value};

use crate::helpers::{LOOPBACK, covers_loopback};
use crate::types::{CaddySource, CaddyVersion, ProxyChoice};

pub const SECTION: &str = "proxy";
pub const NETWORK: &str = "network";
pub const TRUSTED_PROXIES: &str = "trusted_proxies";
pub const MANAGED: &str = "managed";
pub const TLS: &str = "tls";
pub const CADDY: &str = "caddy";

pub fn write_managed(document: &mut DocumentMut, managed: bool) {
    set_keeping(proxy_table(document), MANAGED, managed.into());
}

pub fn write_caddy_source(document: &mut DocumentMut, source: &CaddySource) {
    let wanted = [
        (
            "source",
            (!source.is_default_base()).then(|| source.base.clone()),
        ),
        (
            "version",
            (source.version != CaddyVersion::Latest).then(|| source.version.to_string()),
        ),
    ];
    let nothing = wanted.iter().all(|(_, text)| text.is_none());
    if nothing && !document.contains_table(SECTION) {
        return;
    }
    let table = proxy_table(document);
    if table.get(CADDY).and_then(Item::as_table_like).is_none() {
        if nothing {
            table.remove(CADDY);
            return;
        }
        table[CADDY] = value(InlineTable::new());
    }
    let Some(caddy) = table.get_mut(CADDY).and_then(Item::as_table_like_mut) else {
        return;
    };
    for (key, text) in &wanted {
        match text {
            Some(text) => {
                let mut fresh = Value::from(text.as_str());
                if let Some(existing) = caddy.get(key).and_then(Item::as_value) {
                    *fresh.decor_mut() = existing.decor().clone();
                }
                caddy.insert(key, Item::Value(fresh));
            }
            None => {
                caddy.remove(key);
            }
        }
    }
    if caddy.is_empty() {
        table.remove(CADDY);
    }
}

pub fn write_choice(document: &mut DocumentMut, choice: &ProxyChoice) {
    let table = proxy_table(document);
    set_keeping(table, "enabled", choice.enabled.into());
    let ports = [
        ("http_port", choice.http_port, Publication::HTTP_PORT),
        ("https_port", choice.https_port, Publication::HTTPS_PORT),
    ];
    for (key, port, default) in ports {
        match port.filter(|port| *port != i64::from(default)) {
            Some(port) => set_keeping(table, key, port.into()),
            None => {
                table.remove(key);
            }
        }
    }
    for (key, text) in [
        ("portal_host", &choice.portal_host),
        ("cookie_domain", &choice.cookie_domain),
    ] {
        match text {
            Some(text) => set_keeping(table, key, text.as_str().into()),
            None => {
                table.remove(key);
            }
        }
    }
    match table.get_mut(TLS).and_then(Item::as_table_like_mut) {
        Some(existing) => fill_tls(existing, &choice.tls),
        None => {
            let mut inline = InlineTable::new();
            fill_tls(&mut inline, &choice.tls);
            table[TLS] = value(inline);
        }
    }
}

pub fn trust_loopback(document: &mut DocumentMut) {
    if !document.contains_table(NETWORK) {
        document.insert(NETWORK, Item::Table(Table::new()));
    }
    let network = &mut document[NETWORK];
    let covered = network
        .get(TRUSTED_PROXIES)
        .and_then(Item::as_array)
        .is_some_and(|list| covers_loopback(list.iter().filter_map(|entry| entry.as_str())));
    if covered {
        return;
    }
    match network
        .get_mut(TRUSTED_PROXIES)
        .and_then(Item::as_array_mut)
    {
        Some(list) => list.push(LOOPBACK.to_string()),
        None => {
            let mut list = Array::new();
            list.push(LOOPBACK.to_string());
            network[TRUSTED_PROXIES] = value(list);
        }
    }
}

fn set_keeping(table: &mut Table, key: &str, mut fresh: Value) {
    if let Some(existing) = table.get(key).and_then(Item::as_value) {
        *fresh.decor_mut() = existing.decor().clone();
    }
    table[key] = Item::Value(fresh);
}

fn proxy_table(document: &mut DocumentMut) -> &mut Table {
    if !document.contains_table(SECTION) {
        document.insert(SECTION, Item::Table(Table::new()));
    }
    document[SECTION]
        .as_table_mut()
        .expect("the proxy section was just made a table")
}

fn fill_tls(table: &mut dyn TableLike, tls: &TlsPolicy) {
    table.insert("mode", value(tls.mode.name()));
    let texts = [
        ("email", &tls.email),
        ("certificate", &tls.certificate),
        ("key", &tls.key),
    ];
    for (key, text) in texts {
        match text.as_deref().filter(|text| !text.trim().is_empty()) {
            Some(text) => {
                table.insert(key, value(text));
            }
            None => {
                table.remove(key);
            }
        }
    }
}
