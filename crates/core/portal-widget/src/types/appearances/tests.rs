use serde_json::json;

use super::{Align, WidgetAppearance};

#[test]
fn a_widget_may_align_to_the_end_and_a_wrong_value_names_the_three() {
    let end: WidgetAppearance = serde_json::from_value(json!({ "align": "end" })).unwrap();
    assert_eq!(end.resolved().align, Align::End);
    assert!(end.problems().is_empty());
    let left: WidgetAppearance = serde_json::from_value(json!({ "align": "left" })).unwrap();
    assert_eq!(
        left.problems(),
        vec![(
            "align".to_string(),
            "must be one of start, center, end".to_string()
        )]
    );
}

#[test]
fn twelve_accents_are_accepted_and_a_colour_code_names_them_all() {
    let orange: WidgetAppearance = serde_json::from_value(json!({ "accent": "orange" })).unwrap();
    assert!(orange.problems().is_empty());
    let code: WidgetAppearance = serde_json::from_value(json!({ "accent": "#ff0000" })).unwrap();
    let problems = code.problems();
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].0, "accent");
    for name in [
        "neutral", "blue", "cyan", "teal", "green", "lime", "amber", "orange", "red", "pink",
        "violet", "indigo",
    ] {
        assert!(problems[0].1.contains(name), "{name}: {}", problems[0].1);
    }
}
