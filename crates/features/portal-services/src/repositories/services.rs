use std::path::{Path, PathBuf};

use portal_config::Snapshot;
use toml_edit::{Array, ArrayOfTables, DocumentMut, InlineTable, Item, Table, Value, value};

use portal_model::{Publication, TlsPolicy};

use crate::types::{ProbeSettings, ServiceEntry};

pub const SECTION: &str = "services";

pub fn position(document: &DocumentMut, id: &str) -> Option<usize> {
    document
        .get(SECTION)?
        .as_array_of_tables()?
        .iter()
        .position(|table| table.get("id").and_then(Item::as_str) == Some(id))
}

pub fn published_elsewhere(document: &DocumentMut, entry: &ServiceEntry, own_id: &str) -> bool {
    let Some(host) = entry
        .proxy
        .as_ref()
        .map(|publication| publication.host.as_str())
    else {
        return false;
    };
    let Some(entries) = document.get(SECTION).and_then(Item::as_array_of_tables) else {
        return false;
    };
    entries.iter().any(|table| {
        table.get("id").and_then(Item::as_str) != Some(own_id)
            && table
                .get("proxy")
                .and_then(|proxy| proxy.get("host"))
                .and_then(Item::as_str)
                == Some(host)
    })
}

pub fn append(document: &mut DocumentMut, entry: &ServiceEntry) {
    let mut table = Table::new();
    write_fields(&mut table, entry);
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

pub fn replace(document: &mut DocumentMut, index: usize, entry: &ServiceEntry) {
    if let Some(table) = document
        .get_mut(SECTION)
        .and_then(Item::as_array_of_tables_mut)
        .and_then(|entries| entries.get_mut(index))
    {
        write_fields(table, entry);
    }
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

fn detached_comments(prefix: &str) -> String {
    prefix
        .rfind("\n\n")
        .map(|blank| prefix[..blank + 2].to_string())
        .unwrap_or_default()
}

fn write_fields(table: &mut Table, entry: &ServiceEntry) {
    table["id"] = value(entry.id.as_str());
    table["name"] = value(entry.name.as_str());
    table["url"] = value(entry.url.as_str());
    write_optional(table, "group", entry.group.as_deref());
    write_optional(table, "icon", entry.icon.as_deref());
    write_optional(table, "description", entry.description.as_deref());
    write_addresses(table, entry);
    write_environments(table, entry);
    write_flag(table, "public", entry.public, false);
    write_flag(table, "public_status", entry.public_status, false);
    match entry.notify {
        Some(notify) => table["notify"] = value(notify),
        None => {
            table.remove("notify");
        }
    }
    write_links(table, entry);
    write_notes(table, entry.notes.as_deref());
    write_widgets(table, entry);
    match probe_table(&entry.probe) {
        Some(probe) => table["probe"] = value(probe),
        None => {
            table.remove("probe");
        }
    }
    match &entry.proxy {
        Some(publication) => table["proxy"] = value(publication_table(publication)),
        None => {
            table.remove("proxy");
        }
    }
}

fn publication_table(publication: &Publication) -> InlineTable {
    let mut table = InlineTable::new();
    table.insert("host", publication.host.as_str().into());
    if let Some(upstream) = &publication.upstream {
        table.insert("upstream", upstream.as_str().into());
    }
    if publication.environments != Publication::default_environments() {
        let list: Array = publication
            .environments
            .iter()
            .map(String::as_str)
            .collect();
        table.insert("environments", Value::Array(list));
    }
    if !publication.auth.is_empty() {
        let list: Array = publication.auth.iter().map(String::as_str).collect();
        table.insert("auth", Value::Array(list));
    }
    if let Some(tls) = &publication.tls {
        table.insert("tls", Value::InlineTable(tls_table(tls)));
    }
    if !publication.upstream_verify {
        table.insert("upstream_verify", false.into());
    }
    table
}

fn tls_table(tls: &TlsPolicy) -> InlineTable {
    let mut table = InlineTable::new();
    table.insert("mode", tls.mode.name().into());
    let texts = [
        ("email", &tls.email),
        ("certificate", &tls.certificate),
        ("key", &tls.key),
    ];
    for (key, text) in texts {
        if let Some(text) = text {
            table.insert(key, text.as_str().into());
        }
    }
    table
}

fn write_addresses(table: &mut Table, entry: &ServiceEntry) {
    if entry.addresses.is_empty() {
        table.remove("addresses");
        return;
    }
    let mut addresses = InlineTable::new();
    for (name, address) in &entry.addresses {
        addresses.insert(name, address.as_str().into());
    }
    table["addresses"] = value(addresses);
}

fn write_environments(table: &mut Table, entry: &ServiceEntry) {
    match &entry.environments {
        Some(names) => {
            let list: Array = names.iter().map(String::as_str).collect();
            table["environments"] = value(list);
        }
        None => {
            table.remove("environments");
        }
    }
}

fn write_links(table: &mut Table, entry: &ServiceEntry) {
    if entry.links.is_empty() {
        table.remove("links");
        return;
    }
    let links: Array = entry
        .links
        .iter()
        .map(|link| {
            let mut inline = InlineTable::new();
            inline.insert("title", link.title.as_str().into());
            inline.insert("url", link.url.as_str().into());
            Value::InlineTable(inline)
        })
        .collect();
    table["links"] = value(links);
}

fn write_notes(table: &mut Table, notes: Option<&str>) {
    let Some(notes) = notes.filter(|notes| !notes.trim().is_empty()) else {
        table.remove("notes");
        return;
    };
    let literal_safe = !notes.contains("'''")
        && !notes.ends_with('\'')
        && notes
            .chars()
            .all(|character| !character.is_control() || matches!(character, '\n' | '\t'));
    if !notes.contains('\n') || !literal_safe {
        table["notes"] = value(notes);
        return;
    }
    match format!("'''\n{notes}'''").parse::<Value>() {
        Ok(literal) if literal.as_str() == Some(notes) => table["notes"] = Item::Value(literal),
        _ => table["notes"] = value(notes),
    }
}

fn write_widgets(table: &mut Table, entry: &ServiceEntry) {
    if entry.widgets.is_empty() {
        table.remove("widgets");
        return;
    }
    let widgets: Array = entry.widgets.iter().map(String::as_str).collect();
    table["widgets"] = value(widgets);
}

fn write_flag(table: &mut Table, key: &str, flag: bool, default: bool) {
    if flag == default {
        table.remove(key);
        return;
    }
    table[key] = value(flag);
}

fn write_optional(table: &mut Table, key: &str, text: Option<&str>) {
    match text.filter(|text| !text.is_empty()) {
        Some(text) => table[key] = value(text),
        None => {
            table.remove(key);
        }
    }
}

fn probe_table(probe: &ProbeSettings) -> Option<InlineTable> {
    let defaults = ProbeSettings::default();
    let mut table = InlineTable::new();
    if probe.enabled != defaults.enabled {
        table.insert("enabled", probe.enabled.into());
    }
    if probe.kind != defaults.kind {
        table.insert("kind", probe.kind.name().into());
    }
    if let Some(environment) = &probe.environment {
        table.insert("environment", environment.as_str().into());
    }
    if probe.path != defaults.path {
        table.insert("path", probe.path.as_str().into());
    }
    if let Some(port) = probe.port {
        table.insert("port", i64::from(port).into());
    }
    let numbers = [
        ("every_seconds", probe.every_seconds, defaults.every_seconds),
        (
            "timeout_seconds",
            probe.timeout_seconds,
            defaults.timeout_seconds,
        ),
        (
            "degraded_after_milliseconds",
            probe.degraded_after_milliseconds,
            defaults.degraded_after_milliseconds,
        ),
    ];
    for (key, current, default) in numbers {
        if current != default {
            table.insert(key, i64::try_from(current).unwrap_or(i64::MAX).into());
        }
    }
    (!table.is_empty()).then_some(table)
}

pub fn origin(snapshot: &Snapshot, id: &str) -> Option<PathBuf> {
    let index = position(&snapshot.document, id)?;
    snapshot.origins.of(SECTION, index).map(Path::to_path_buf)
}
