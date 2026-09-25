use portal_widget::{SectionEntry, WidgetInstance, WidgetSize, WidgetsSection};
use toml_edit::DocumentMut;

use super::write_layout;
use crate::responses::WidgetView;
use crate::types::{EditedLayout, EditedWidget};

const BEFORE: &str = r#"# the home page

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

# a widget of a newer portal
[[dashboard.widgets]]
type = "traffic"
id = "roads"
settings = { city = "Riga", zoom = 12, layers = ["jams", "works"] }
"#;

fn document() -> DocumentMut {
    BEFORE.parse().unwrap()
}

fn current(document: &DocumentMut) -> EditedLayout {
    let layout = WidgetsSection::layout(document).unwrap().unwrap();
    EditedLayout {
        sections: layout.sections,
        widgets: layout
            .widgets
            .into_iter()
            .enumerate()
            .map(|(index, instance)| EditedWidget {
                key: Some(WidgetView::key_of(index, &instance)),
                instance,
            })
            .collect(),
    }
}

fn written(edit: impl FnOnce(&mut EditedLayout)) -> String {
    let mut document = document();
    let mut edited = current(&document);
    edit(&mut edited);
    write_layout(&mut document, &edited);
    document.to_string()
}

#[test]
fn saving_an_unchanged_layout_changes_nothing_but_giving_ids() {
    let text = written(|_| {});
    assert_eq!(
        text,
        BEFORE.replace(
            "type = \"status-summary\"\n",
            "type = \"status-summary\"\nid = \"status-summary\"\n"
        )
    );
}

#[test]
fn a_moved_widget_takes_its_comment_along_and_the_rest_of_the_file_stays() {
    let text = written(|edited| {
        let first = edited.widgets.remove(0);
        edited.widgets.push(first);
    });
    let expected = r#"# the home page

[network]
port = 8080

# the weather, public
[[dashboard.widgets]]
type = "weather"
id = "riga"
public = true
settings = { latitude = 56.95, longitude = 24.11 }

# a widget of a newer portal
[[dashboard.widgets]]
type = "traffic"
id = "roads"
settings = { city = "Riga", zoom = 12, layers = ["jams", "works"] }

# counts first
[[dashboard.widgets]]
type = "status-summary"
id = "status-summary"
"#;
    assert_eq!(text, expected);
}

#[test]
fn an_unknown_type_keeps_its_settings_line_byte_for_byte() {
    let text = written(|edited| edited.widgets.swap(0, 1));
    assert!(
        text.contains(
            "settings = { city = \"Riga\", zoom = 12, layers = [\"jams\", \"works\"] }\n"
        )
    );
}

#[test]
fn a_new_widget_gets_an_id_unique_in_the_layout_and_only_the_keys_it_needs() {
    let text = written(|edited| {
        edited.widgets.push(EditedWidget {
            key: None,
            instance: WidgetInstance {
                size: WidgetSize::Half,
                ..WidgetInstance::of("status-summary")
            },
        });
    });
    assert!(text.ends_with(
        "\n[[dashboard.widgets]]\ntype = \"status-summary\"\nid = \"status-summary-2\"\nsize = \"half\"\n"
    ), "{text}");
}

#[test]
fn a_removed_widget_is_gone_and_its_neighbours_keep_their_comments() {
    let text = written(|edited| {
        edited.widgets.remove(1);
    });
    assert!(!text.contains("riga"));
    assert!(text.contains("# counts first\n[[dashboard.widgets]]"));
    assert!(text.contains("# a widget of a newer portal\n[[dashboard.widgets]]"));
}

#[test]
fn a_changed_size_and_title_touch_only_those_keys() {
    let text = written(|edited| {
        edited.widgets[1].instance.size = WidgetSize::Third;
        edited.widgets[1].instance.title = Some("Weather".into());
    });
    assert!(text.contains(
        "type = \"weather\"\nid = \"riga\"\npublic = true\nsettings = { latitude = 56.95, longitude = 24.11 }\ntitle = \"Weather\"\nsize = \"third\"\n"
    ), "{text}");
}

#[test]
fn sections_are_written_only_when_there_is_more_than_the_implicit_one() {
    let text = written(|edited| {
        edited.sections = vec![
            SectionEntry {
                id: "now".into(),
                title: Some("Now".into()),
            },
            SectionEntry {
                id: "later".into(),
                title: None,
            },
        ];
        edited.widgets[2].instance.section = Some("later".into());
        edited.widgets[0].instance.section = Some("now".into());
        edited.widgets[1].instance.section = Some("now".into());
    });
    assert!(
        text.contains("[[dashboard.sections]]\nid = \"now\"\ntitle = \"Now\"\n"),
        "{text}"
    );
    assert!(text.contains("id = \"roads\"\nsettings = { city = \"Riga\", zoom = 12, layers = [\"jams\", \"works\"] }\nsection = \"later\"\n"), "{text}");
    let reread: DocumentMut = text.parse().unwrap();
    let layout = WidgetsSection::layout(&reread).unwrap().unwrap();
    assert_eq!(layout.widgets[0].section.as_deref(), Some("now"));
    assert_eq!(layout.widgets[2].section.as_deref(), Some("later"));
    let untouched = written(|_| {});
    assert!(!untouched.contains("sections"));
    assert!(!untouched.contains("section ="));
}
