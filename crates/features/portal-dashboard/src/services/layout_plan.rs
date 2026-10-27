use std::collections::HashMap;

use portal_widget::{SectionAppearance, SectionEntry, WidgetInstance};

use crate::types::{EditedLayout, LayoutPlan, PlacementKey, PlannedEntry};

pub fn layout_plan(
    stored: &[WidgetInstance],
    sections: &[SectionEntry],
    edited: &EditedLayout,
) -> LayoutPlan {
    let implicit = edited.only_the_implicit_section() && sections.is_empty();
    let first_section = sections
        .first()
        .map(|section| section.id.clone())
        .unwrap_or_else(|| SectionEntry::IMPLICIT.to_string());
    let mut by_key: HashMap<String, usize> = (0..stored.len())
        .map(|index| (PlacementKey::of(index), index))
        .collect();
    let placements = edited
        .widgets
        .iter()
        .map(|widget| {
            let mut wanted = widget.instance.clone();
            if implicit {
                wanted.section = None;
            }
            let index = widget.key.as_ref().and_then(|key| by_key.remove(key));
            let mut old = index
                .map(|index| stored[index].clone())
                .unwrap_or_else(|| WidgetInstance::of(""));
            if old.section.is_none() && !implicit {
                old.section = Some(first_section.clone());
            }
            if wanted.widget.is_none() {
                wanted.widget = old.widget.clone();
            }
            PlannedEntry {
                stored: index,
                old,
                wanted,
            }
        })
        .collect();
    LayoutPlan {
        placements,
        sections: if implicit {
            Vec::new()
        } else {
            planned_sections(sections, edited)
        },
    }
}

fn planned_sections(
    sections: &[SectionEntry],
    edited: &EditedLayout,
) -> Vec<PlannedEntry<SectionEntry>> {
    let mut by_id: HashMap<String, usize> = sections
        .iter()
        .enumerate()
        .map(|(index, section)| (section.id.clone(), index))
        .collect();
    edited
        .sections
        .iter()
        .map(|section| {
            let index = by_id.remove(&section.id);
            let old = index
                .map(|index| sections[index].clone())
                .unwrap_or_else(|| SectionEntry {
                    id: String::new(),
                    title: None,
                    appearance: SectionAppearance::default(),
                });
            PlannedEntry {
                stored: index,
                old,
                wanted: section.clone(),
            }
        })
        .collect()
}
