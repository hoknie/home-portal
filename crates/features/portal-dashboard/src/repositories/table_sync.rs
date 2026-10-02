use portal_widget::{WidgetHeight, WidgetInstance};
use toml_edit::{Array, InlineTable, Item, Table, value};

use crate::helpers::{renamed_in_place, toml_of};

pub const SIZE: &str = "size";
pub const WIDTH: &str = "width";
pub const APPEARANCE: &str = "appearance";
pub const WIDGET_LOOK: [&str; 5] = ["surface", "accent", "title", "padding", "align"];
pub const SECTION_LOOK: [&str; 2] = ["title", "surface"];

pub fn sync<T: PartialEq>(
    table: &mut Table,
    key: &str,
    old: &T,
    wanted: &T,
    item: impl FnOnce() -> Option<Item>,
) {
    if old == wanted {
        return;
    }
    match item() {
        Some(item) => {
            table.insert(key, item);
        }
        None => {
            table.remove(key);
        }
    }
}

pub fn appearance_item<T: serde::Serialize>(appearance: &T, order: &[&str]) -> Option<Item> {
    let Ok(serde_json::Value::Object(fields)) = serde_json::to_value(appearance) else {
        return None;
    };
    let mut table = InlineTable::new();
    for key in order.iter().copied() {
        if let Some(value) = fields.get(key).and_then(toml_of) {
            table.insert(key, value);
        }
    }
    (!table.is_empty()).then(|| Item::Value(toml_edit::Value::InlineTable(table)))
}

pub fn write_definition(table: &mut Table, old: &WidgetInstance, wanted: &WidgetInstance) {
    sync(table, "id", &old.id, &wanted.id, || {
        wanted.id.as_deref().map(value)
    });
    sync(table, "type", &old.kind, &wanted.kind, || {
        Some(value(wanted.kind.as_str()))
    });
    sync(table, "title", &old.title, &wanted.title, || {
        wanted.title.as_deref().map(value)
    });
    sync(
        table,
        APPEARANCE,
        &old.appearance,
        &wanted.appearance,
        || appearance_item(&wanted.appearance, &WIDGET_LOOK),
    );
    sync(
        table,
        "environments",
        &old.environments,
        &wanted.environments,
        || {
            wanted.environments.as_ref().map(|names| {
                let list: Array = names.iter().map(String::as_str).collect();
                value(list)
            })
        },
    );
    sync(table, "public", &old.public, &wanted.public, || {
        wanted.public.then(|| value(true))
    });
    let empty = wanted
        .settings
        .as_object()
        .is_none_or(serde_json::Map::is_empty);
    sync(table, "settings", &old.settings, &wanted.settings, || {
        if empty {
            None
        } else {
            toml_of(&wanted.settings).map(Item::Value)
        }
    });
    write_width(table, old, wanted);
    sync(
        table,
        "height",
        &old.height,
        &wanted.height,
        || match wanted.height {
            WidgetHeight::Rows(rows) => Some(value(i64::from(rows))),
            WidgetHeight::Auto | WidgetHeight::Invalid => None,
        },
    );
}

pub fn write_placement(table: &mut Table, old: &WidgetInstance, wanted: &WidgetInstance) {
    sync(table, "widget", &old.widget, &wanted.widget, || {
        wanted.widget.as_deref().map(value)
    });
    sync(table, "section", &old.section, &wanted.section, || {
        wanted.section.as_deref().map(value)
    });
    write_width(table, old, wanted);
    sync(
        table,
        "height",
        &old.height,
        &wanted.height,
        || match wanted.height {
            WidgetHeight::Rows(rows) => Some(value(i64::from(rows))),
            WidgetHeight::Auto | WidgetHeight::Invalid => None,
        },
    );
    sync(table, "column", &old.column, &wanted.column, || {
        wanted.column.map(value)
    });
    sync(table, "row", &old.row, &wanted.row, || {
        wanted.row.map(value)
    });
}

fn write_width(table: &mut Table, old: &WidgetInstance, wanted: &WidgetInstance) {
    let columns = wanted.columns();
    if table.contains_key(SIZE) {
        renamed_in_place(table, SIZE, WIDTH, value(i64::from(columns)));
        return;
    }
    sync(table, WIDTH, &old.columns(), &columns, || {
        (columns != WidgetInstance::COLUMNS).then(|| value(i64::from(columns)))
    });
}
