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
    let text = "[[dashboard.sections]]\nid = \"now\"\ntitle = \"Сейчас\"\n\n[[dashboard.sections]]\nid = \"media\"\n\n[[dashboard.widgets]]\ntype = \"status-summary\"\n\n[[dashboard.widgets]]\ntype = \"services\"\nsection = \"media\"\nsize = \"half\"\n";
    let layout = layout(&document(text)).unwrap();
    assert_eq!(layout.widgets[0].section.as_deref(), Some("now"));
    assert_eq!(layout.widgets[0].size.name(), "full");
    assert_eq!(layout.widgets[1].section.as_deref(), Some("media"));
    assert_eq!(layout.widgets[1].size.columns(), 6);
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
