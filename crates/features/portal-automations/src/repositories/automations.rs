use std::path::PathBuf;

use portal_config::Snapshot;
use toml_edit::{Array, DocumentMut, InlineTable, Table, Value};

use super::{push, remove_at, set, set_nested, table_at, tags_value};
use crate::types::{Automation, AutomationsSection, Filters, RunSettings, StateFilter, Trigger};

pub const SECTION: &str = AutomationsSection::SECTION;

pub fn position(document: &DocumentMut, id: &str) -> Option<usize> {
    super::tables::position(document, SECTION, id)
}

pub fn origin(snapshot: &Snapshot, id: &str) -> Option<PathBuf> {
    super::tables::origin(snapshot, SECTION, id)
}

pub fn append(document: &mut DocumentMut, automation: &Automation) {
    let mut table = Table::new();
    write_fields(&mut table, automation);
    push(document, SECTION, table);
}

pub fn replace(document: &mut DocumentMut, index: usize, automation: &Automation) {
    if let Some(table) = table_at(document, SECTION, index) {
        write_fields(table, automation);
    }
}

pub fn remove(document: &mut DocumentMut, index: usize) {
    remove_at(document, SECTION, index);
}

fn write_fields(table: &mut Table, automation: &Automation) {
    set(table, "id", Some(automation.id.as_str().into()));
    set(table, "title", Some(automation.title.as_str().into()));
    set(
        table,
        "enabled",
        (!automation.enabled).then(|| false.into()),
    );
    set(table, "tags", tags_value(&automation.tags));
    set(
        table,
        "cooldown_seconds",
        (automation.cooldown_seconds != 0).then(|| (automation.cooldown_seconds as i64).into()),
    );
    set_nested(table, "when", when_table(&automation.trigger));
    set_nested(table, "run", run_table(&automation.run));
}

fn when_table(trigger: &Trigger) -> InlineTable {
    let mut table = InlineTable::new();
    table.insert("event", trigger.event.name().into());
    let Filters {
        services,
        states,
        users,
        environments,
        webhooks,
        cron,
    } = &trigger.filters;
    let StateFilter {
        from,
        to,
        from_unknown,
    } = states;
    if let Some(cron) = cron {
        table.insert("cron", cron.expression.as_str().into());
    }
    for (key, list) in [
        ("services", services),
        ("from", from),
        ("to", to),
        ("users", users),
        ("environments", environments),
        ("webhooks", webhooks),
    ] {
        if !list.is_empty() {
            let items: Array = list.iter().map(String::as_str).collect();
            table.insert(key, Value::Array(items));
        }
    }
    if *from_unknown {
        table.insert("from_unknown", true.into());
    }
    table
}

pub fn run_table(run: &RunSettings) -> InlineTable {
    let mut table = InlineTable::new();
    table.insert("script", run.script.as_str().into());
    if !run.args.is_empty() {
        let items: Array = run.args.iter().map(String::as_str).collect();
        table.insert("args", Value::Array(items));
    }
    if run.timeout_seconds != RunSettings::DEFAULT_TIMEOUT {
        table.insert("timeout_seconds", (run.timeout_seconds as i64).into());
    }
    table
}
