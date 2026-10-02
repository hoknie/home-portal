use std::collections::BTreeMap;

use serde_json::json;

use super::rendering::{LONGEST_TEXT, cut, web_address};
use super::tones::{leading_number, number_in, tone_for};
use crate::types::{Tone, ToneFields};

fn thresholds(warning: f64, danger: f64) -> ToneFields {
    serde_json::from_value(json!({ "thresholds": { "warning": warning, "danger": danger } }))
        .unwrap()
}

#[test]
fn a_number_past_its_thresholds_takes_their_tone_and_text_takes_the_tone_it_names() {
    let disk = thresholds(75.0, 90.0);
    assert_eq!(tone_for(&disk, "93"), Tone::Danger);
    assert_eq!(tone_for(&disk, "80 %"), Tone::Warning);
    assert_eq!(tone_for(&disk, "12"), Tone::Ok);
    assert_eq!(tone_for(&disk, "unknown"), Tone::Neutral);
    let states = ToneFields {
        tones: Some(BTreeMap::from([
            ("ok".to_string(), Tone::Ok),
            ("failed".to_string(), Tone::Danger),
        ])),
        ..ToneFields::default()
    };
    assert_eq!(tone_for(&states, "failed"), Tone::Danger);
    assert_eq!(tone_for(&states, "running"), Tone::Neutral);
    let below: ToneFields = serde_json::from_value(
        json!({ "thresholds": { "warning": 20, "danger": 10, "direction": "below" } }),
    )
    .unwrap();
    assert_eq!(tone_for(&below, "8"), Tone::Danger);
    assert_eq!(tone_for(&below, "50"), Tone::Ok);
}

#[test]
fn numbers_are_read_from_values_and_from_the_start_of_a_text() {
    assert_eq!(number_in(&json!(93)), Some(93.0));
    assert_eq!(number_in(&json!("-4.5 °C")), Some(-4.5));
    assert_eq!(leading_number("GB"), None);
}

#[test]
fn a_long_text_is_cut_with_an_ellipsis_and_only_web_addresses_are_links() {
    let long = "x".repeat(LONGEST_TEXT + 5);
    let shown = cut(long);
    assert_eq!(shown.chars().count(), LONGEST_TEXT);
    assert!(shown.ends_with('…'));
    assert!(web_address("https://nas.home.lan"));
    assert!(!web_address("ftp://nas"));
    assert!(!web_address("javascript:alert(1)"));
}

#[test]
fn an_answer_larger_than_sixty_four_kilobytes_fails_as_too_much_content() {
    use super::rendering::{TOO_MUCH, WidgetMeta, render_widget};
    use crate::types::{Block, CustomWidget};
    let list: Block = serde_json::from_value(json!({
        "kind": "list", "items": "{{data.rows}}", "text": "{{item}}", "secondary": "{{item}}", "limit": 50
    }))
    .unwrap();
    let widget = CustomWidget {
        source: None,
        refresh: std::time::Duration::from_secs(300),
        blocks: vec![list],
        actions: Vec::new(),
    };
    let rows: Vec<String> = (0..50).map(|_| "x".repeat(900)).collect();
    let meta = WidgetMeta {
        id: "big",
        title: None,
        fetched_at: None,
    };
    assert_eq!(
        render_widget(
            &crate::fakes::FakeTemplates,
            &widget,
            &json!({ "rows": rows }),
            &meta
        ),
        Err(TOO_MUCH.to_string())
    );
    let few: Vec<String> = (0..3).map(|_| "x".repeat(900)).collect();
    assert!(
        render_widget(
            &crate::fakes::FakeTemplates,
            &widget,
            &json!({ "rows": few }),
            &meta
        )
        .is_ok()
    );
}

#[test]
fn a_block_takes_its_own_align_else_its_nearest_column_and_groups_keep_their_nesting() {
    use super::rendering::{WidgetMeta, render_widget};
    use crate::responses::CustomWidgetData;
    use crate::types::{Block, CustomWidget};
    let row: Block = serde_json::from_value(json!({
        "kind": "row", "widths": [8, 4], "row_align": "center", "blocks": [
            { "kind": "stat", "label": "Free", "value": "{{data.free}}" },
            { "kind": "column", "align": "end", "valign": "center", "blocks": [
                { "kind": "text", "text": "a", "valign": "end" },
                { "kind": "text", "text": "b", "align": "start" }
            ] }
        ]
    }))
    .unwrap();
    let widget = CustomWidget {
        source: None,
        refresh: std::time::Duration::from_secs(300),
        blocks: vec![row],
        actions: Vec::new(),
    };
    let meta = WidgetMeta {
        id: "disks",
        title: None,
        fetched_at: None,
    };
    let blocks = render_widget(
        &crate::fakes::FakeTemplates,
        &widget,
        &json!({ "free": 120 }),
        &meta,
    )
    .unwrap();
    let answer = serde_json::to_value(CustomWidgetData { blocks }).unwrap();
    let row = &answer["blocks"][0];
    assert_eq!(row["align"], "center");
    assert_eq!(row["widths"], json!([8, 4]));
    assert_eq!(row["blocks"][0]["align"], json!(null));
    assert_eq!(row["blocks"][0]["value"], "120");
    let column = &row["blocks"][1];
    assert_eq!(column["align"], "end");
    assert_eq!(column["valign"], "center");
    assert_eq!(column["blocks"][0]["valign"], "end");
    assert_eq!(column["blocks"][0]["align"], "end");
    assert_eq!(column["blocks"][1]["align"], "start");
}
