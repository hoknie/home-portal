use portal_feature::{FieldError, Principal, WidgetProvider};
use serde_json::{Value, json};

use super::support::{data_of, widgets_api};

fn checked(settings: Value) -> Vec<String> {
    let api = widgets_api(Principal::admin("admin"));
    api.feature
        .custom
        .check(&settings)
        .into_iter()
        .map(|error: FieldError| format!("{}: {}", error.field, error.message))
        .collect()
}

#[tokio::test]
async fn a_workflow_source_gives_its_vars_and_the_blocks_render_them_without_the_data() {
    let api = widgets_api(Principal::admin("admin"));
    let rendered = data_of(&api, "disks").await.unwrap();
    assert_eq!(rendered["blocks"][0]["value"], "120 of 500 GB");
    assert_eq!(rendered["blocks"][1]["kind"], "button");
    assert_eq!(rendered["blocks"][1]["action"], "restart");
    let text = rendered.to_string();
    assert!(!text.contains("abc"), "{text}");
    assert!(!text.contains("restart-media"), "{text}");
}

#[tokio::test]
async fn a_script_source_reads_json_or_text_and_a_list_renders_each_item() {
    let api = widgets_api(Principal::admin("admin"));
    let clients = data_of(&api, "clients").await.unwrap();
    assert_eq!(
        clients["blocks"][0]["items"][1],
        json!({ "text": "tv", "secondary": "10.0.0.3", "tone": "neutral" })
    );
    let backup = data_of(&api, "backup").await.unwrap();
    assert_eq!(backup["blocks"][0]["text"], "backup ok, 3 h ago");
}

#[tokio::test]
async fn static_content_runs_nothing_and_a_failing_source_names_the_failure_in_general_words() {
    let api = widgets_api(Principal::admin("admin"));
    let wifi = data_of(&api, "wifi").await.unwrap();
    assert_eq!(wifi["blocks"][0]["text"], "Wi-Fi: home-5G");
    assert_eq!(
        data_of(&api, "failing").await.unwrap_err(),
        "the workflow failed"
    );
    assert_eq!(*api.runs.sources.lock().unwrap(), ["broken"]);
}

#[tokio::test]
async fn a_public_answer_keeps_the_blocks_but_drops_the_buttons() {
    let api = widgets_api(Principal::admin("admin"));
    let backup = data_of(&api, "backup").await.unwrap();
    let public = api.feature.custom.public_data(backup);
    assert_eq!(public["blocks"].as_array().unwrap().len(), 1);
}

#[test]
fn bad_settings_are_named_by_their_place_in_the_widget() {
    let errors = checked(
        json!({ "source": { "workflow": "nope" }, "blocks": [{ "kind": "text", "text": "{{steps.probe.state}}" }] }),
    );
    assert!(
        errors
            .iter()
            .any(|error| error.starts_with("source.workflow:")),
        "{errors:?}"
    );
    assert!(
        errors
            .iter()
            .any(|error| error.starts_with("blocks[0].text:") && error.contains("widget.title")),
        "{errors:?}"
    );
    let outside = checked(
        json!({ "source": { "script": "../secret.sh" }, "blocks": [{ "kind": "divider" }] }),
    );
    assert!(
        outside
            .iter()
            .any(|error| error.starts_with("source.script:")),
        "{outside:?}"
    );
    let refresh = checked(json!({ "refresh_seconds": 5, "blocks": [{ "kind": "divider" }] }));
    assert!(
        refresh
            .iter()
            .any(|error| error.starts_with("refresh_seconds:")),
        "{refresh:?}"
    );
    let tones = checked(
        json!({ "blocks": [{ "kind": "badge", "text": "x", "tone": "ok", "tones": { "x": "danger" } }] }),
    );
    assert!(
        tones
            .iter()
            .any(|error| error.starts_with("blocks[0].tone:")),
        "{tones:?}"
    );
}

#[test]
fn a_widget_holds_at_most_forty_blocks_with_the_nested_ones_and_a_row_two_to_four() {
    let many: Vec<Value> = (0..30).map(|_| json!({ "kind": "divider" })).collect();
    let mut blocks = vec![json!({ "kind": "row", "blocks": many })];
    blocks.extend((0..15).map(|_| json!({ "kind": "divider" })));
    let errors = checked(json!({ "blocks": blocks }));
    assert!(
        errors
            .iter()
            .any(|error| error.starts_with("blocks:") && error.contains("40")),
        "{errors:?}"
    );
    assert!(
        errors
            .iter()
            .any(|error| error.starts_with("blocks[0].blocks:") && error.contains("from 2 to 4")),
        "{errors:?}"
    );
    let two = || json!([{ "kind": "divider" }, { "kind": "divider" }]);
    let nested = checked(json!({ "blocks": [{ "kind": "row", "blocks": [
        { "kind": "column", "blocks": [{ "kind": "row", "blocks": [
            { "kind": "column", "blocks": [{ "kind": "text", "text": "x" }] }, { "kind": "divider" }
        ] }] },
        { "kind": "divider" }
    ] }] }));
    assert!(
        nested.iter().any(
            |error| error.starts_with("blocks[0].blocks[0].blocks[0].blocks[0]:")
                && error.contains("at most 3 deep")
        ),
        "{nested:?}"
    );
    let allowed = checked(
        json!({ "blocks": [{ "kind": "row", "widths": [8, 4], "align": "center", "blocks": [
        { "kind": "stat", "label": "Free", "value": "{{data.free}}" },
        { "kind": "column", "align": "end", "blocks": [{ "kind": "row", "blocks": two() }] }
    ] }] }),
    );
    assert!(allowed.is_empty(), "{allowed:?}");
    let wrong = checked(
        json!({ "blocks": [{ "kind": "row", "widths": [12], "align": "middle", "blocks": two() },
        { "kind": "text", "text": "x", "align": "left" }] }),
    );
    for field in ["blocks[0].widths:", "blocks[0].align:", "blocks[1].align:"] {
        assert!(
            wrong.iter().any(|error| error.starts_with(field)),
            "{field} {wrong:?}"
        );
    }
}
