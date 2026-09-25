use std::path::PathBuf;

use portal_config::Snapshot;
use toml_edit::{Array, DocumentMut, Table, Value};

use super::{push, remove_at, run_table, set, set_nested, table_at, tags_value};
use crate::types::{AutomationsSection, Webhook, WebhookAction};

pub const SECTION: &str = AutomationsSection::WEBHOOKS;
pub const TOKEN_KEY: &str = "token_sha256";

pub fn webhook_position(document: &DocumentMut, id: &str) -> Option<usize> {
    super::tables::position(document, SECTION, id)
}

pub fn webhook_origin(snapshot: &Snapshot, id: &str) -> Option<PathBuf> {
    super::tables::origin(snapshot, SECTION, id)
}

pub fn append_webhook(document: &mut DocumentMut, webhook: &Webhook) {
    let mut table = Table::new();
    write_fields(&mut table, webhook);
    set(
        &mut table,
        TOKEN_KEY,
        webhook.token_sha256.as_deref().map(Value::from),
    );
    push(document, SECTION, table);
}

pub fn replace_webhook(document: &mut DocumentMut, index: usize, webhook: &Webhook) {
    if let Some(table) = table_at(document, SECTION, index) {
        write_fields(table, webhook);
    }
}

pub fn remove_webhook(document: &mut DocumentMut, index: usize) {
    remove_at(document, SECTION, index);
}

pub fn set_token(document: &mut DocumentMut, index: usize, hash: Option<&str>) {
    if let Some(table) = table_at(document, SECTION, index) {
        set(table, TOKEN_KEY, hash.map(Value::from));
    }
}

fn write_fields(table: &mut Table, webhook: &Webhook) {
    set(table, "id", Some(webhook.id.as_str().into()));
    set(table, "title", Some(webhook.title.as_str().into()));
    set(table, "enabled", (!webhook.enabled).then(|| false.into()));
    set(table, "tags", tags_value(&webhook.tags));
    let variables: Array = webhook.variables.iter().map(String::as_str).collect();
    set(
        table,
        "variables",
        (!webhook.variables.is_empty()).then_some(Value::Array(variables)),
    );
    set(table, "action", Some(webhook.action.name().into()));
    match &webhook.action {
        WebhookAction::Event => {
            table.remove("run");
        }
        WebhookAction::Script(run) => set_nested(table, "run", run_table(run)),
    }
}
