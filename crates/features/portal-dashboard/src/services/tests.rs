use toml_edit::DocumentMut;

use super::{layout, validate_dashboard};

fn document(text: &str) -> DocumentMut {
    text.parse().unwrap()
}

fn fields(text: &str) -> Vec<String> {
    validate_dashboard(&document(text))
        .into_iter()
        .map(|error| error.field)
        .collect()
}

#[test]
fn without_a_dashboard_section_the_default_layout_is_a_summary_then_the_services() {
    let layout = layout(&document("")).unwrap();
    let kinds: Vec<String> = layout
        .widgets
        .into_iter()
        .map(|widget| widget.kind)
        .collect();
    assert_eq!(kinds, vec!["status-summary", "services"]);
    assert_eq!(layout.sections.len(), 1);
    assert_eq!(layout.sections[0].id, "main");
}

#[test]
fn widgets_keep_the_file_order_and_their_settings() {
    let text = "[[dashboard.widgets]]\ntype = \"services\"\ntitle = \"Media\"\nsettings = { groups = [\"Media\"] }\n\n[[dashboard.widgets]]\ntype = \"status-summary\"\n";
    let widgets = layout(&document(text)).unwrap().widgets;
    assert_eq!(widgets[0].kind, "services");
    assert_eq!(widgets[0].title.as_deref(), Some("Media"));
    assert_eq!(widgets[0].settings["groups"][0], "Media");
    assert_eq!(widgets[1].kind, "status-summary");
}

#[test]
fn a_widget_of_an_unknown_type_is_passed_through_unchanged() {
    let text = "[[dashboard.widgets]]\ntype = \"weather\"\nid = \"riga\"\nsettings = { city = \"Riga\" }\n";
    let widgets = layout(&document(text)).unwrap().widgets;
    assert_eq!(widgets[0].kind, "weather");
    assert_eq!(widgets[0].id.as_deref(), Some("riga"));
    assert_eq!(widgets[0].settings["city"], "Riga");
    assert!(validate_dashboard(&document(text)).is_empty());
}

#[test]
fn a_widget_without_a_type_is_refused() {
    assert_eq!(
        fields("[[dashboard.widgets]]\ntype = \"\"\n"),
        vec!["dashboard.widgets[0].type"]
    );
}

#[test]
fn a_widget_without_a_section_belongs_to_the_first_one_and_is_full_width() {
    let text = "[[dashboard.sections]]\nid = \"now\"\ntitle = \"Now\"\n\n[[dashboard.sections]]\nid = \"media\"\n\n[[dashboard.widgets]]\ntype = \"status-summary\"\n\n[[dashboard.widgets]]\ntype = \"services\"\nsection = \"media\"\nsize = \"half\"\n";
    let layout = layout(&document(text)).unwrap();
    assert_eq!(layout.widgets[0].section.as_deref(), Some("now"));
    assert_eq!(layout.widgets[0].columns(), 12);
    assert_eq!(layout.widgets[1].section.as_deref(), Some("media"));
    assert_eq!(layout.widgets[1].columns(), 6);
    assert!(validate_dashboard(&document(text)).is_empty());
}

#[test]
fn a_misspelled_size_is_refused_naming_the_widget_and_the_allowed_values() {
    let text = "[[dashboard.widgets]]\ntype = \"status-summary\"\n\n[[dashboard.widgets]]\ntype = \"services\"\n\n[[dashboard.widgets]]\ntype = \"services\"\nsize = \"wide\"\n";
    let errors = validate_dashboard(&document(text));
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].field, "dashboard.widgets[2].size");
    assert!(errors[0].message.contains("two-thirds"));
}

#[test]
fn a_widget_naming_a_section_that_does_not_exist_is_refused() {
    assert_eq!(
        fields("[[dashboard.widgets]]\ntype = \"services\"\nsection = \"media\"\n"),
        vec!["dashboard.widgets[0].section"]
    );
    assert_eq!(
        fields(
            "[[dashboard.sections]]\nid = \"now\"\n\n[[dashboard.widgets]]\ntype = \"services\"\nsection = \"media\"\n"
        ),
        vec!["dashboard.widgets[0].section"]
    );
}

#[test]
fn two_sections_with_one_id_or_a_bad_id_are_refused() {
    assert_eq!(
        fields(
            "[[dashboard.sections]]\nid = \"now\"\n\n[[dashboard.sections]]\nid = \"now\"\n\n[[dashboard.sections]]\nid = \"Bad Id\"\n"
        ),
        vec!["dashboard.sections[1].id", "dashboard.sections[2].id"]
    );
}

fn calendar(url: &str, secret: &str) -> portal_widget::WidgetInstance {
    let mut widget = portal_widget::WidgetInstance::of("calendar");
    widget.settings = serde_json::json!({ "url": url, "secret": secret });
    widget
}

fn edited(widgets: Vec<portal_widget::WidgetInstance>) -> crate::types::EditedLayout {
    crate::types::EditedLayout {
        sections: Vec::new(),
        widgets: widgets
            .into_iter()
            .map(|instance| crate::types::EditedWidget {
                key: None,
                instance,
            })
            .collect(),
    }
}

#[test]
fn a_layout_editor_cannot_point_a_secret_somewhere_new_but_may_keep_one() {
    use portal_feature::{Action, Area, Right, Rights};
    let editor = Rights::of([Right::new(Area::Layout, Action::Update)]);
    let stored = vec![calendar("https://cloud.home/cal.ics", "nas_token")];
    let borrowed = edited(vec![calendar("https://example.net/", "nas_token")]);
    let refused = super::secrets_allowed(&borrowed, &stored, &editor).unwrap_err();
    assert!(
        format!("{refused:?}").contains("widgets[0].secret"),
        "{refused:?}"
    );
    let kept = edited(vec![calendar("https://cloud.home/cal.ics", "nas_token")]);
    assert!(super::secrets_allowed(&kept, &stored, &editor).is_ok());
    let reader = Rights::of([Right::new(Area::Secrets, Action::Read)]);
    assert!(super::secrets_allowed(&borrowed, &stored, &reader).is_ok());
}

#[test]
fn a_width_is_read_and_a_size_from_an_older_file_gives_its_columns() {
    let text = "[[dashboard.widgets]]\ntype = \"services\"\nwidth = 5\nheight = 3\n\n[[dashboard.widgets]]\ntype = \"services\"\nsize = \"third\"\nheight = \"auto\"\n";
    let widgets = layout(&document(text)).unwrap().widgets;
    assert_eq!(widgets[0].columns(), 5);
    assert_eq!(widgets[0].height, portal_widget::WidgetHeight::Rows(3));
    assert_eq!(widgets[1].columns(), 4);
    assert!(widgets[1].height.is_auto());
    assert!(validate_dashboard(&document(text)).is_empty());
}

#[test]
fn widths_and_heights_out_of_range_and_both_width_and_size_are_refused_by_field() {
    assert_eq!(
        fields("[[dashboard.widgets]]\ntype = \"services\"\nwidth = 13\n"),
        vec!["dashboard.widgets[0].width"]
    );
    assert_eq!(
        fields(
            "[[dashboard.widgets]]\ntype = \"services\"\nheight = 9\n\n[[dashboard.widgets]]\ntype = \"services\"\nheight = \"tall\"\n"
        ),
        vec!["dashboard.widgets[0].height", "dashboard.widgets[1].height"]
    );
    let both = validate_dashboard(&document(
        "[[dashboard.widgets]]\ntype = \"services\"\nwidth = 6\nsize = \"half\"\n",
    ));
    assert_eq!(both[0].field, "dashboard.widgets[0].width");
    assert!(both[0].message.contains("size"));
}

#[test]
fn a_colour_code_or_an_unknown_key_in_a_look_is_refused_naming_the_allowed_values() {
    let errors = validate_dashboard(&document(
        "[[dashboard.sections]]\nid = \"now\"\nappearance = { surface = \"glass\" }\n\n[[dashboard.widgets]]\ntype = \"services\"\nappearance = { accent = \"#ff0000\", shadow = true }\n",
    ));
    let named: Vec<&str> = errors.iter().map(|error| error.field.as_str()).collect();
    assert_eq!(
        named,
        vec![
            "dashboard.sections[0].appearance.surface",
            "dashboard.widgets[0].appearance.accent",
            "dashboard.widgets[0].appearance.shadow"
        ]
    );
    assert!(errors[1].message.contains("violet"));
    assert!(!errors[1].message.ends_with(", "));
}

fn custom(id: &str, source: &str) -> portal_widget::WidgetInstance {
    let mut widget = portal_widget::WidgetInstance::of("custom");
    widget.id = Some(id.into());
    widget.settings = serde_json::json!({ "source": source, "title": "anything" });
    widget
}

fn scripted() -> crate::types::NeedsOf {
    use portal_feature::{Action, Area, NeedScope, Right, WidgetNeed};
    std::sync::Arc::new(|kind: &str, settings: &serde_json::Value| {
        if kind != "custom" {
            return Vec::new();
        }
        let source = settings["source"].as_str().unwrap_or_default().to_string();
        vec![WidgetNeed::new(
            "source",
            Right::new(Area::Automations, Action::Execute),
            source,
            NeedScope::SameWidget,
        )]
    })
}

fn keyed(
    widgets: Vec<(Option<&str>, portal_widget::WidgetInstance)>,
) -> crate::types::EditedLayout {
    crate::types::EditedLayout {
        sections: Vec::new(),
        widgets: widgets
            .into_iter()
            .map(|(key, instance)| crate::types::EditedWidget {
                key: key.map(String::from),
                instance,
            })
            .collect(),
    }
}

#[test]
fn a_new_source_needs_its_right_but_a_widget_keeps_the_one_it_had_through_other_changes() {
    use portal_feature::{Action, Area, Right, Rights};
    let editor = Rights::of([Right::new(Area::Layout, Action::Update)]);
    let stored = vec![custom("disks", "router-clients.sh")];
    let added = keyed(vec![(None, custom("clients", "router-clients.sh"))]);
    let refused = super::needs_allowed(&added, &stored, &editor, &scripted()).unwrap_err();
    assert!(
        format!("{refused:?}").contains("widgets[0].source"),
        "{refused:?}"
    );
    let mut restyled = custom("disks", "router-clients.sh");
    restyled.width = Some(5);
    restyled.settings["title"] = serde_json::json!("changed");
    let kept = keyed(vec![(Some("disks"), restyled)]);
    assert!(super::needs_allowed(&kept, &stored, &editor, &scripted()).is_ok());
    let swapped = keyed(vec![(Some("disks"), custom("disks", "other.sh"))]);
    assert!(super::needs_allowed(&swapped, &stored, &editor, &scripted()).is_err());
    let runner = Rights::of([Right::new(Area::Automations, Action::Execute)]);
    assert!(super::needs_allowed(&added, &stored, &runner, &scripted()).is_ok());
}

#[test]
fn a_place_names_a_library_widget_and_carries_only_its_place() {
    let text = "[[dashboard.library]]\nid = \"riga\"\ntype = \"weather\"\nsection = \"main\"\n\n[[dashboard.widgets]]\nwidget = \"riga\"\ntitle = \"Riga\"\n\n[[dashboard.widgets]]\nwidget = \"nope\"\n";
    assert_eq!(
        fields(text),
        vec![
            "dashboard.library[0].section",
            "dashboard.widgets[0].title",
            "dashboard.widgets[1].widget"
        ]
    );
}

#[test]
fn a_position_is_a_column_and_a_row_together_and_fits_the_twelve_columns() {
    let library = "[[dashboard.library]]\nid = \"riga\"\ntype = \"weather\"\n\n";
    assert!(
        fields(&format!(
            "{library}[[dashboard.widgets]]\nwidget = \"riga\"\ncolumn = 9\nrow = 2\nwidth = 4\n"
        ))
        .is_empty()
    );
    assert_eq!(
        fields(&format!(
            "{library}[[dashboard.widgets]]\nwidget = \"riga\"\ncolumn = 10\nrow = 1\nwidth = 4\n\n[[dashboard.widgets]]\nwidget = \"riga\"\ncolumn = 1\n"
        )),
        vec!["dashboard.widgets[0].column", "dashboard.widgets[1].column"]
    );
}
