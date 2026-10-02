use portal_widget::{
    Accent, SectionAppearance, SectionEntry, SectionSurface, Surface, TitleVisibility,
    WidgetAppearance, WidgetHeight, WidgetInstance, WidgetsSection,
};
use toml_edit::DocumentMut;

use super::{remove_library_entry, write_layout, write_library_entry};
use crate::responses::WidgetView;
use crate::types::{EditedLayout, EditedWidget};

const OLDER: &str = r#"# the home page

[network]
port = 8080

# counts first
[[dashboard.widgets]]
type = "status-summary"

# the weather, public
[[dashboard.widgets]]
type = "weather"
id = "riga"
public = true
settings = { latitude = 56.95, longitude = 24.11 }
"#;

fn current(document: &DocumentMut) -> EditedLayout {
    let layout = WidgetsSection::layout(document).unwrap().unwrap();
    EditedLayout {
        sections: layout.sections,
        widgets: layout
            .widgets
            .into_iter()
            .enumerate()
            .map(|(index, widget)| EditedWidget {
                key: Some(WidgetView::key_of(index)),
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

fn written(text: &str, edit: impl FnOnce(&mut EditedLayout)) -> String {
    let mut document: DocumentMut = text.parse().unwrap();
    let mut edited = current(&document);
    edit(&mut edited);
    write_layout(&mut document, &edited);
    document.to_string()
}

#[test]
fn saving_an_older_layout_moves_each_definition_into_the_library_and_keeps_the_comments_on_the_places()
 {
    let text = written(OLDER, |_| {});
    let expected = r#"# the home page

[network]
port = 8080

# counts first
[[dashboard.widgets]]
widget = "status-summary"

# the weather, public
[[dashboard.widgets]]
widget = "riga"

[[dashboard.library]]
id = "status-summary"
type = "status-summary"

[[dashboard.library]]
id = "riga"
type = "weather"
public = true
settings = { latitude = 56.95, longitude = 24.11 }
"#;
    assert_eq!(text, expected);
    let reread: DocumentMut = text.parse().unwrap();
    let layout = WidgetsSection::layout(&reread).unwrap().unwrap();
    assert_eq!(layout.widgets[1].settings["latitude"], 56.95);
    assert!(layout.widgets[1].public);
}

#[test]
fn a_moved_place_takes_its_comment_along() {
    let text = written(OLDER, |edited| {
        let first = edited.widgets.remove(0);
        edited.widgets.push(first);
    });
    assert!(text.contains("# the weather, public\n[[dashboard.widgets]]\nwidget = \"riga\"\n\n# counts first\n[[dashboard.widgets]]\nwidget = \"status-summary\"\n"), "{text}");
}

#[test]
fn a_position_and_a_size_are_written_on_the_place_only() {
    let text = written(OLDER, |edited| {
        edited.widgets[1].instance.column = Some(9);
        edited.widgets[1].instance.row = Some(2);
        edited.widgets[1].instance.width = Some(4);
        edited.widgets[1].instance.height = WidgetHeight::Rows(3);
    });
    assert!(
        text.contains("widget = \"riga\"\nwidth = 4\nheight = 3\ncolumn = 9\nrow = 2\n"),
        "{text}"
    );
    let back = written(&text, |edited| {
        edited.widgets[1].instance.column = None;
        edited.widgets[1].instance.row = None;
    });
    assert!(!back.contains("column"), "{back}");
}

#[test]
fn a_library_widget_can_be_placed_twice() {
    let text = written(OLDER, |edited| {
        edited.widgets.push(EditedWidget {
            key: None,
            instance: WidgetInstance::placed("riga", "main"),
        });
    });
    assert_eq!(text.matches("widget = \"riga\"").count(), 2, "{text}");
}

#[test]
fn an_older_size_is_rewritten_as_a_width_in_its_place_with_its_comment() {
    let sized = "[[dashboard.widgets]]\nwidget = \"riga\"\n# half the row\nsize = \"half\"\n\n[[dashboard.library]]\nid = \"riga\"\ntype = \"weather\"\n";
    let text = written(sized, |edited| {
        edited.widgets[0].instance.width = Some(5);
        edited.widgets[0].instance.size = None;
    });
    assert!(
        text.starts_with("[[dashboard.widgets]]\nwidget = \"riga\"\n# half the row\nwidth = 5\n"),
        "{text}"
    );
}

#[test]
fn a_section_look_is_written_without_its_defaults() {
    let text = written(OLDER, |edited| {
        edited.sections = vec![
            SectionEntry {
                id: "now".into(),
                title: Some("Now".into()),
                appearance: SectionAppearance {
                    title: Some(TitleVisibility::Hidden),
                    surface: Some(SectionSurface::SectionCard),
                    ..SectionAppearance::default()
                },
            },
            SectionEntry {
                id: "later".into(),
                title: None,
                appearance: SectionAppearance::default(),
            },
        ];
        for widget in &mut edited.widgets {
            widget.instance.section = Some("now".into());
        }
    });
    assert!(text.contains("[[dashboard.sections]]\nid = \"now\"\ntitle = \"Now\"\nappearance = { title = \"hidden\", surface = \"card\" }\n"), "{text}");
}

#[test]
fn a_library_entry_is_added_with_a_derived_id_changed_in_place_and_removed() {
    let mut document: DocumentMut = OLDER.parse().unwrap();
    let id = write_library_entry(&mut document, None, &WidgetInstance::of("weather"));
    assert_eq!(id, "weather");
    let changed = WidgetInstance {
        title: Some("Riga".into()),
        appearance: WidgetAppearance {
            surface: Some(Surface::Tinted),
            accent: Some(Accent::Green),
            ..WidgetAppearance::default()
        },
        ..WidgetInstance::of("weather")
    };
    write_library_entry(&mut document, Some("riga"), &changed);
    let text = document.to_string();
    assert!(text.contains("title = \"Riga\"\n"), "{text}");
    assert!(
        text.contains("appearance = { surface = \"tinted\", accent = \"green\" }\n"),
        "{text}"
    );
    assert!(!text.contains("public = true"), "{text}");
    assert!(
        text.contains("[[dashboard.library]]\nid = \"weather\"\ntype = \"weather\"\n"),
        "{text}"
    );
    remove_library_entry(&mut document, "weather");
    assert!(!document.to_string().contains("id = \"weather\""));
}

#[test]
fn the_first_write_to_a_file_without_a_layout_keeps_the_default_widgets_in_the_library() {
    let mut document: DocumentMut = "[network]\nport = 8080\n".parse().unwrap();
    let edited = EditedLayout {
        sections: vec![SectionEntry::implicit()],
        widgets: vec![EditedWidget {
            key: None,
            instance: WidgetInstance::placed("services", SectionEntry::IMPLICIT),
        }],
    };
    write_layout(&mut document, &edited);
    let library = WidgetsSection::library(&document).unwrap();
    let ids: Vec<_> = library
        .iter()
        .filter_map(|entry| entry.id.as_deref())
        .collect();
    assert_eq!(ids, ["status-summary", "services"]);
    let placements = WidgetsSection::placements(&document).unwrap();
    assert_eq!(placements.len(), 1);
    assert_eq!(placements[0].widget.as_deref(), Some("services"));
}

#[test]
fn a_library_widget_keeps_its_own_size_and_a_wrong_one_is_named() {
    let mut document: DocumentMut = "[network]\nport = 8080\n".parse().unwrap();
    let wanted = WidgetInstance {
        id: Some("disks".into()),
        width: Some(4),
        height: WidgetHeight::Rows(2),
        ..WidgetInstance::of("custom")
    };
    write_library_entry(&mut document, None, &wanted);
    let text = document.to_string();
    assert!(text.contains("width = 4"), "{text}");
    assert!(text.contains("height = 2"), "{text}");
    let library = WidgetsSection::library(&document).unwrap();
    let disks = library
        .iter()
        .find(|entry| entry.id.as_deref() == Some("disks"))
        .unwrap();
    assert_eq!(disks.columns(), 4, "{text}");
    assert_eq!(disks.height, WidgetHeight::Rows(2));
    let wrong = crate::services::check_entry(&WidgetInstance {
        width: Some(13),
        ..WidgetInstance::of("custom")
    });
    assert!(
        wrong.iter().any(|error| error.field == "width"),
        "{wrong:?}"
    );
}
