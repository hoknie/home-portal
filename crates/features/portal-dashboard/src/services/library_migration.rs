use portal_widget::{WidgetInstance, WidgetsSection};

use super::DEFAULT_WIDGETS;
use crate::types::{InlinePlacement, LibraryMoves, StoredWidgets};

pub fn library_moves(stored: &StoredWidgets) -> LibraryMoves {
    let defaults: Vec<String> = if stored.dashboard {
        Vec::new()
    } else {
        DEFAULT_WIDGETS
            .iter()
            .map(|kind| kind.to_string())
            .collect()
    };
    let inline: Vec<InlinePlacement> = if stored.dashboard {
        stored.inline.clone()
    } else {
        defaults
            .iter()
            .enumerate()
            .map(|(index, kind)| InlinePlacement {
                index,
                kind: kind.clone(),
                id: None,
            })
            .collect()
    };
    let mut taken: Vec<String> = stored.library_ids.clone();
    taken.extend(inline.iter().filter_map(|placement| placement.id.clone()));
    let moves = inline
        .iter()
        .map(|placement| {
            let id = match &placement.id {
                Some(id) => id.clone(),
                None => {
                    let id = WidgetsSection::derived_id(&placement.kind, &taken);
                    taken.push(id.clone());
                    id
                }
            };
            (placement.index, id)
        })
        .collect();
    LibraryMoves { defaults, moves }
}

pub fn library_entry_id(
    existing: Option<&str>,
    wanted: &WidgetInstance,
    library: &[WidgetInstance],
) -> String {
    match (existing, &wanted.id) {
        (Some(id), _) => id.to_string(),
        (None, Some(id)) => id.clone(),
        (None, None) => {
            let taken: Vec<String> = library
                .iter()
                .filter_map(|entry| entry.id.clone())
                .collect();
            WidgetsSection::derived_id(&wanted.kind, &taken)
        }
    }
}
