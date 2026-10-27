use portal_widget::WidgetInstance;
use toml_edit::DocumentMut;

use crate::repositories::{
    apply_layout, apply_library_moves, remove_library_entry as remove_entry, stored_layout,
    stored_library, stored_widgets, write_library_entry as write_entry,
};
use crate::services::{layout_plan, library_entry_id, library_moves};
use crate::types::EditedLayout;

fn migrated(document: &mut DocumentMut) {
    apply_library_moves(document, &library_moves(&stored_widgets(document)));
}

pub fn write_layout(document: &mut DocumentMut, edited: &EditedLayout) {
    migrated(document);
    let (placements, sections) = stored_layout(document);
    apply_layout(document, &layout_plan(&placements, &sections, edited));
}

pub fn write_library_entry(
    document: &mut DocumentMut,
    existing: Option<&str>,
    wanted: &WidgetInstance,
) -> String {
    migrated(document);
    let id = library_entry_id(existing, wanted, &stored_library(document));
    let written = WidgetInstance {
        id: Some(id.clone()),
        ..wanted.clone()
    };
    write_entry(document, existing, &written);
    id
}

pub fn remove_library_entry(document: &mut DocumentMut, id: &str) {
    migrated(document);
    remove_entry(document, id);
}
