use portal_widget::{SectionEntry, WidgetHeight, WidgetInstance, WidgetsSection};
use toml_edit::DocumentMut;

use super::golden_expected::EXPECTED;
use super::steps::{remove_library_entry, write_layout, write_library_entry};
use crate::types::{EditedLayout, EditedWidget, PlacementKey};

const IMPLICIT: &str = r#"# the home page
[network]
port = 8080

# counts first
[[dashboard.widgets]]
widget = "status-summary"

[[dashboard.widgets]]
widget = "riga"
width = 6

[[dashboard.library]]
id = "status-summary"
type = "status-summary"

# the weather
[[dashboard.library]]
id = "riga"
type = "weather"
settings = { latitude = 56.95, longitude = 24.11 }
"#;

const SECTIONED: &str = r#"[[dashboard.sections]]
id = "now"
title = "Now"

# later things
[[dashboard.sections]]
id = "later"

# no section of its own
[[dashboard.widgets]]
widget = "status-summary"

[[dashboard.widgets]]
widget = "riga"
section = "later"

[[dashboard.library]]
id = "status-summary"
type = "status-summary"

[[dashboard.library]]
id = "riga"
type = "weather"
"#;

const INLINE: &str = r#"[[dashboard.library]]
id = "weather"
type = "weather"

# an inline clock
[[dashboard.widgets]]
type = "clock"
title = "Time"

# an inline weather without an id
[[dashboard.widgets]]
type = "weather"
settings = { latitude = 1.5 }

[[dashboard.widgets]]
widget = "weather"
"#;

fn edited_from(document: &DocumentMut) -> EditedLayout {
    let Some(layout) = WidgetsSection::layout(document).unwrap() else {
        return EditedLayout {
            sections: Vec::new(),
            widgets: Vec::new(),
        };
    };
    EditedLayout {
        sections: layout.sections,
        widgets: layout
            .widgets
            .into_iter()
            .enumerate()
            .map(|(index, widget)| EditedWidget {
                key: Some(PlacementKey::of(index)),
                instance: WidgetInstance {
                    column: widget.column,
                    row: widget.row,
                    width: widget.width,
                    size: widget.size,
                    height: widget.height,
                    ..WidgetInstance::placed(
                        widget.id.as_deref().unwrap_or_default(),
                        widget.section.as_deref().unwrap_or_default(),
                    )
                },
            })
            .collect(),
    }
}

fn layout_written(text: &str, edit: impl FnOnce(&mut EditedLayout)) -> String {
    let mut document: DocumentMut = text.parse().unwrap();
    let mut edited = edited_from(&document);
    edit(&mut edited);
    write_layout(&mut document, &edited);
    document.to_string()
}

fn library_written(
    text: &str,
    existing: Option<&str>,
    wanted: &WidgetInstance,
) -> (String, String) {
    let mut document: DocumentMut = text.parse().unwrap();
    let id = write_library_entry(&mut document, existing, wanted);
    (id, document.to_string())
}

fn cases() -> Vec<(&'static str, String)> {
    let mut cases = vec![
        ("implicit_unchanged", layout_written(IMPLICIT, |_| {})),
        (
            "implicit_reordered_and_resized",
            layout_written(IMPLICIT, |edited| {
                let first = edited.widgets.remove(0);
                edited.widgets.push(first);
                edited.widgets[0].instance.width = Some(4);
                edited.widgets[0].instance.height = WidgetHeight::Rows(2);
            }),
        ),
        (
            "implicit_becomes_sections",
            layout_written(IMPLICIT, |edited| {
                edited.sections = vec![SectionEntry {
                    id: "main".into(),
                    title: Some("Main".into()),
                    ..SectionEntry::implicit()
                }];
                edited.widgets[1].instance.section = Some("main".into());
            }),
        ),
        ("sectioned_unchanged", layout_written(SECTIONED, |_| {})),
        (
            "sectioned_widget_without_section_moved",
            layout_written(SECTIONED, |edited| {
                edited.widgets[0].instance.column = Some(3);
                edited.widgets[0].instance.section = None;
            }),
        ),
        (
            "sectioned_edit_omits_widget",
            layout_written(SECTIONED, |edited| {
                edited.widgets[1].instance.widget = None;
                edited.widgets[1].instance.row = Some(5);
            }),
        ),
        (
            "sectioned_new_placement",
            layout_written(SECTIONED, |edited| {
                edited.widgets.push(EditedWidget {
                    key: None,
                    instance: WidgetInstance::placed("riga", "now"),
                });
                edited.sections.pop();
            }),
        ),
        ("inline_migrated", layout_written(INLINE, |_| {})),
        (
            "absent_dashboard",
            layout_written("[network]\nport = 8080\n", |edited| {
                edited.sections = vec![SectionEntry::implicit()];
                edited.widgets = vec![EditedWidget {
                    key: None,
                    instance: WidgetInstance::placed("services", SectionEntry::IMPLICIT),
                }];
            }),
        ),
    ];
    for (name, existing, wanted) in [
        ("library_new_derived", None, WidgetInstance::of("weather")),
        (
            "library_new_given",
            None,
            WidgetInstance {
                id: Some("disks".into()),
                title: Some("Disks".into()),
                width: Some(4),
                ..WidgetInstance::of("custom")
            },
        ),
        (
            "library_replaced",
            Some("riga"),
            WidgetInstance {
                title: Some("Riga".into()),
                ..WidgetInstance::of("weather")
            },
        ),
    ] {
        let (id, text) = library_written(IMPLICIT, existing, &wanted);
        cases.push((name, format!("{id}\n---\n{text}")));
    }
    let (id, text) = library_written(INLINE, None, &WidgetInstance::of("clock"));
    cases.push(("library_new_after_migration", format!("{id}\n---\n{text}")));
    let mut document: DocumentMut = INLINE.parse().unwrap();
    remove_library_entry(&mut document, "weather");
    cases.push(("library_removed_after_migration", document.to_string()));
    cases
}

#[test]
fn layout_and_library_writes_keep_their_bytes() {
    let cases = cases();
    assert_eq!(cases.len(), EXPECTED.len());
    for ((name, text), (expected_name, expected)) in cases.iter().zip(EXPECTED) {
        assert_eq!(*name, expected_name);
        assert_eq!(text, expected, "{name}");
    }
}
