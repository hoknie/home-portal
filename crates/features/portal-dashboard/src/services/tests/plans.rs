use portal_widget::{SectionEntry, WidgetInstance};

use crate::services::{layout_plan, library_entry_id, library_moves};
use crate::types::{EditedLayout, EditedWidget, InlinePlacement, PlacementKey, StoredWidgets};

fn placed(widget: &str, section: Option<&str>) -> WidgetInstance {
    WidgetInstance {
        widget: Some(widget.into()),
        section: section.map(String::from),
        ..WidgetInstance::of("")
    }
}

fn section(id: &str) -> SectionEntry {
    SectionEntry {
        id: id.into(),
        ..SectionEntry::implicit()
    }
}

fn edited(
    sections: Vec<SectionEntry>,
    widgets: Vec<(Option<usize>, WidgetInstance)>,
) -> EditedLayout {
    EditedLayout {
        sections,
        widgets: widgets
            .into_iter()
            .map(|(index, instance)| EditedWidget {
                key: index.map(PlacementKey::of),
                instance,
            })
            .collect(),
    }
}

#[test]
fn with_only_the_implicit_section_no_section_is_written_or_kept() {
    let stored = [placed("riga", Some("main"))];
    let plan = layout_plan(
        &stored,
        &[],
        &edited(
            vec![SectionEntry::implicit()],
            vec![(Some(0), placed("riga", Some("main")))],
        ),
    );
    assert_eq!(plan.placements[0].wanted.section, None);
    assert_eq!(plan.placements[0].stored, Some(0));
    assert!(plan.sections.is_empty());
}

#[test]
fn a_stored_widget_without_a_section_counts_as_in_the_first_section() {
    let stored = [placed("riga", None)];
    let sections = [section("now"), section("later")];
    let plan = layout_plan(
        &stored,
        &sections,
        &edited(
            sections.to_vec(),
            vec![(Some(0), placed("riga", Some("now")))],
        ),
    );
    assert_eq!(plan.placements[0].old.section.as_deref(), Some("now"));
    assert_eq!(plan.sections.len(), 2);
    assert_eq!(plan.sections[1].stored, Some(1));
}

#[test]
fn an_edit_without_a_widget_keeps_the_stored_reference() {
    let stored = [placed("riga", None)];
    let wanted = WidgetInstance {
        widget: None,
        ..placed("", None)
    };
    let plan = layout_plan(&stored, &[], &edited(vec![], vec![(Some(0), wanted)]));
    assert_eq!(plan.placements[0].wanted.widget.as_deref(), Some("riga"));
}

#[test]
fn a_key_is_used_once_and_an_unknown_key_is_a_new_placement() {
    let stored = [placed("riga", None)];
    let plan = layout_plan(
        &stored,
        &[],
        &edited(
            vec![],
            vec![
                (Some(0), placed("riga", None)),
                (Some(0), placed("riga", None)),
                (Some(7), placed("riga", None)),
            ],
        ),
    );
    let stored: Vec<_> = plan.placements.iter().map(|entry| entry.stored).collect();
    assert_eq!(stored, [Some(0), None, None]);
}

#[test]
fn without_a_dashboard_the_default_widgets_move_into_the_library() {
    let moves = library_moves(&StoredWidgets {
        dashboard: false,
        library_ids: Vec::new(),
        inline: Vec::new(),
    });
    assert_eq!(moves.defaults, ["status-summary", "services"]);
    assert_eq!(
        moves.moves,
        [
            (0, "status-summary".to_string()),
            (1, "services".to_string())
        ]
    );
}

#[test]
fn an_inline_widget_keeps_its_id_or_gets_a_free_one() {
    let moves = library_moves(&StoredWidgets {
        dashboard: true,
        library_ids: vec!["weather".into()],
        inline: vec![
            InlinePlacement {
                index: 0,
                kind: "weather".into(),
                id: None,
            },
            InlinePlacement {
                index: 2,
                kind: "clock".into(),
                id: Some("time".into()),
            },
        ],
    });
    assert!(moves.defaults.is_empty());
    assert_eq!(
        moves.moves,
        [(0, "weather-2".to_string()), (2, "time".to_string())]
    );
}

#[test]
fn a_library_entry_keeps_the_edited_id_then_its_own_then_a_free_one() {
    let library = [WidgetInstance {
        id: Some("clock".into()),
        ..WidgetInstance::of("clock")
    }];
    let named = WidgetInstance {
        id: Some("time".into()),
        ..WidgetInstance::of("clock")
    };
    assert_eq!(library_entry_id(Some("clock"), &named, &library), "clock");
    assert_eq!(library_entry_id(None, &named, &library), "time");
    assert_eq!(
        library_entry_id(None, &WidgetInstance::of("clock"), &library),
        "clock-2"
    );
}
