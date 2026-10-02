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
