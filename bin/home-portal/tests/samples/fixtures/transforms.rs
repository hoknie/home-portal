use portal_automations::{PreviewQuestion, RawOperation, TransformPreviewResponse, TransformValue};
use serde_json::json;

use crate::typed;

#[test]
fn the_transform_preview_sample_matches_its_serializer() {
    let operations: Vec<RawOperation> = serde_json::from_value(json!([
        {"op": "filter", "where": {"left": "{{item.health}}", "op": "!=", "right": "ok"}},
        {"op": "pluck", "args": ["name"]},
        {"op": "number"}
    ]))
    .unwrap();
    let value = json!({"disks": [
        {"name": "sda", "health": "ok"},
        {"name": "sdb", "health": "failing"}
    ]});
    let examples = vec!["length".to_string(), "pluck(\"name\")".to_string()];
    let preview = TransformValue
        .run(&PreviewQuestion {
            value,
            filters: "| get(\"disks\")".into(),
            operations,
            examples,
            ..PreviewQuestion::default()
        })
        .unwrap();
    typed("transform-preview", &TransformPreviewResponse::of(&preview));
}
