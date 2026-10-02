use std::path::PathBuf;

use portal_config::Snapshot;
use toml_edit::{Array, ArrayOfTables, DocumentMut, Item, Table, Value};

use super::{push, remove_at, set, table_at, tags_value};
use crate::types::{InputDeclaration, OutputDeclaration, RawStep, Workflow};

pub const WORKFLOWS: &str = Workflow::SECTION;
pub const STEP_LISTS: [&str; 3] = ["then", "else", "body"];
pub const KEY_ORDER: [&str; 43] = [
    "id",
    "label",
    "kind",
    "condition",
    "repeat",
    "for_each",
    "while",
    "max_iterations",
    "branches",
    "workflow",
    "inputs",
    "outcome",
    "reason",
    "variable",
    "value",
    "json",
    "list",
    "object",
    "seconds",
    "method",
    "url",
    "headers",
    "timeout_seconds",
    "fail_on_error",
    "response_sample",
    "script",
    "args",
    "env",
    "stdin",
    "title",
    "text",
    "channel",
    "message",
    "level",
    "service",
    "automation",
    "fields",
    "wait",
    "input",
    "operations",
    "then",
    "else",
    "body",
];

pub fn workflow_position(document: &DocumentMut, id: &str) -> Option<usize> {
    super::tables::position(document, WORKFLOWS, id)
}

pub fn workflow_origin(snapshot: &Snapshot, id: &str) -> Option<PathBuf> {
    super::tables::origin(snapshot, WORKFLOWS, id)
}

pub fn append_workflow(document: &mut DocumentMut, workflow: &Workflow, steps: &[RawStep]) {
    let mut table = Table::new();
    write_workflow(&mut table, workflow, steps);
    push(document, WORKFLOWS, table);
}

pub fn replace_workflow(
    document: &mut DocumentMut,
    index: usize,
    workflow: &Workflow,
    steps: &[RawStep],
) {
    if let Some(table) = table_at(document, WORKFLOWS, index) {
        write_workflow(table, workflow, steps);
    }
}

pub fn remove_workflow(document: &mut DocumentMut, index: usize) {
    remove_at(document, WORKFLOWS, index);
}

fn write_workflow(table: &mut Table, workflow: &Workflow, steps: &[RawStep]) {
    set(table, "id", Some(workflow.id.as_str().into()));
    set(table, "title", Some(workflow.title.as_str().into()));
    set(table, "enabled", (!workflow.enabled).then(|| false.into()));
    set(
        table,
        "description",
        workflow
            .description
            .as_deref()
            .map(|description| description.into()),
    );
    set(table, "tags", tags_value(&workflow.tags));
    set(
        table,
        "timeout_seconds",
        (workflow.timeout_seconds != Workflow::DEFAULT_TIMEOUT)
            .then(|| (workflow.timeout_seconds as i64).into()),
    );
    set(
        table,
        "inputs",
        (!workflow.inputs.is_empty()).then(|| {
            Value::Array(
                workflow
                    .inputs
                    .iter()
                    .filter_map(input_value)
                    .collect::<Array>(),
            )
        }),
    );
    set(
        table,
        OutputDeclaration::FIELD,
        (!workflow.outputs.is_empty())
            .then(|| Value::Array(workflow.outputs.iter().map(output_value).collect::<Array>())),
    );
    table.insert("steps", steps_item(steps));
}

fn output_value(output: &OutputDeclaration) -> Value {
    let mut inline = toml_edit::InlineTable::new();
    inline.insert("name", output.name.as_str().into());
    inline.insert("value", output.value.as_str().into());
    if let Some(description) = &output.description {
        inline.insert("description", description.as_str().into());
    }
    Value::InlineTable(inline)
}

fn steps_item(steps: &[RawStep]) -> Item {
    let tables: Vec<toml::Table> = steps
        .iter()
        .filter_map(|step| toml::Table::try_from(step).ok())
        .collect();
    list_item(&tables)
}

fn list_item(tables: &[toml::Table]) -> Item {
    if tables.is_empty() {
        return Item::Value(Value::Array(Array::new()));
    }
    let mut list = ArrayOfTables::new();
    for step in tables {
        list.push(step_table(step));
    }
    Item::ArrayOfTables(list)
}

fn step_table(step: &toml::Table) -> Table {
    let mut table = Table::new();
    let ordered = KEY_ORDER
        .iter()
        .filter_map(|key| step.get_key_value(*key))
        .chain(
            step.iter()
                .filter(|(key, _)| !KEY_ORDER.contains(&key.as_str())),
        );
    for (key, entry) in ordered {
        match nested_steps(key, entry) {
            Some(tables) => {
                table.insert(key, list_item(&tables));
            }
            None => {
                if let Some(value) = inline(entry) {
                    table.insert(key, Item::Value(value));
                }
            }
        }
    }
    table
}

fn nested_steps(key: &str, entry: &toml::Value) -> Option<Vec<toml::Table>> {
    if !STEP_LISTS.contains(&key) {
        return None;
    }
    let list = entry.as_array()?;
    list.iter()
        .map(|item| item.as_table().cloned())
        .collect::<Option<Vec<_>>>()
}

fn input_value(input: &InputDeclaration) -> Option<Value> {
    if input.is_plain() {
        return Some(input.name.as_str().into());
    }
    let mut table = toml_edit::InlineTable::new();
    table.insert("name", input.name.as_str().into());
    table.insert("type", input.input_type.name().into());
    if let Some(default) = input.default.as_ref().and_then(json_value) {
        table.insert("default", default);
    }
    if let Some(description) = &input.description {
        table.insert("description", description.as_str().into());
    }
    Some(Value::InlineTable(table))
}

pub fn json_value(value: &serde_json::Value) -> Option<Value> {
    toml::Value::try_from(value).ok().as_ref().and_then(inline)
}

fn inline(entry: &toml::Value) -> Option<Value> {
    let mut wrapper = toml::Table::new();
    wrapper.insert("v".into(), entry.clone());
    let text = toml::to_string(&wrapper).ok()?;
    let parsed: DocumentMut = text.parse().ok()?;
    match parsed.get("v")? {
        Item::Value(value) => {
            let mut value = value.clone();
            value.decor_mut().clear();
            Some(value)
        }
        Item::Table(table) => Some(Value::InlineTable(table.clone().into_inline_table())),
        Item::ArrayOfTables(tables) => Some(Value::Array(
            tables
                .iter()
                .map(|table| {
                    let mut inline = table.clone().into_inline_table();
                    inline.fmt();
                    Value::InlineTable(inline)
                })
                .collect(),
        )),
        _ => None,
    }
}
