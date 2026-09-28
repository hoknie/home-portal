use crate::services::workflow as super_workflow;

fn frame_with(
    steps: &[(&str, serde_json::Value)],
    secrets: &[(&str, &str)],
) -> super_workflow::Frame {
    let owned: Vec<(String, String)> = secrets
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect();
    let lookup: super_workflow::SecretLookup = std::sync::Arc::new(move |key: &str| {
        owned
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.clone())
    });
    let mut frame = super_workflow::Frame::new(
        std::sync::Arc::new(vec![("service.id".to_string(), "nas".to_string())]),
        [("service".to_string(), serde_json::json!("jellyfin"))].into(),
        std::sync::Arc::new(super_workflow::Secrets::new(lookup)),
    );
    for (id, value) in steps {
        frame.steps.insert(id.to_string(), value.clone());
    }
    frame
}

#[test]
fn reading_a_json_answer_by_key_and_index() {
    let frame = frame_with(
        &[(
            "state",
            serde_json::json!({"json": {"disks": [{"health": "ok"}], "usedPercent": 91.5}}),
        )],
        &[],
    );
    assert_eq!(
        super_workflow::render_text("disk: {{steps.state.json.disks.0.health}}", &frame).unwrap(),
        "disk: ok"
    );
    assert_eq!(
        super_workflow::render_text(
            "{{steps.state.json.usedPercent}}% {{event.service.id}} {{inputs.service}}",
            &frame
        )
        .unwrap(),
        "91.5% nas jellyfin"
    );
    assert_eq!(
        super_workflow::render_text("[{{steps.state.json.missing.0}}]", &frame).unwrap(),
        "[]"
    );
}

#[test]
fn a_single_placeholder_keeps_its_json_value() {
    let frame = frame_with(&[("list", serde_json::json!({"json": [1, 2, 3]}))], &[]);
    assert_eq!(
        super_workflow::render_value("{{steps.list.json}}", &frame).unwrap(),
        serde_json::json!([1, 2, 3])
    );
    assert_eq!(
        super_workflow::render_value("list: {{steps.list.json}}", &frame).unwrap(),
        serde_json::json!("list: [1,2,3]")
    );
    assert_eq!(
        super_workflow::render_json("{\"a\": {{steps.list.json.1}}}", &frame).unwrap(),
        serde_json::json!({"a": 2})
    );
}

#[test]
fn a_secret_is_sent_as_it_is_and_masked_everywhere_after() {
    let frame = frame_with(&[], &[("nas_token", "s3cret-value")]);
    let header = super_workflow::render_text("Bearer {{secrets.nas_token}}", &frame).unwrap();
    assert_eq!(header, "Bearer s3cret-value");
    assert_eq!(frame.secrets.mask(&header), "Bearer ***");
    assert_eq!(frame.secrets.mask("echoed: s3cret-value!"), "echoed: ***!");
    assert!(super_workflow::render_text("{{secrets.missing}}", &frame).is_err());
}

#[test]
fn a_rendering_larger_than_64_kib_is_refused() {
    let big = "x".repeat(40 * 1024);
    let frame = frame_with(&[("big", serde_json::json!(big))], &[]);
    assert!(super_workflow::render_text("{{steps.big}}", &frame).is_ok());
    assert!(super_workflow::render_text("{{steps.big}}{{steps.big}}", &frame).is_err());
}

#[test]
fn every_operator_compares_as_the_spec_says() {
    use crate::types::Operator::*;
    use serde_json::json;
    let cases: Vec<(serde_json::Value, crate::types::Operator, &str, bool)> = vec![
        (json!("91.5"), Greater, "90", true),
        (json!(91.5), GreaterOrEqual, "91.5", true),
        (json!("9"), Less, "10", true),
        (json!("abc"), Less, "10", false),
        (json!("10"), LessOrEqual, "9", false),
        (json!(200), Equal, "200", true),
        (json!("up"), NotEqual, "up", false),
        (json!("disk full"), Contains, "full", true),
        (json!(["a", "b"]), Contains, "b", true),
        (json!(["a", "b"]), Contains, "c", false),
        (json!(null), IsEmpty, "", true),
        (json!("  "), IsEmpty, "", true),
        (json!([]), IsEmpty, "", true),
        (json!({"a": 1}), IsNotEmpty, "", true),
        (json!(0), IsEmpty, "", false),
    ];
    for (left, operator, right, expected) in cases {
        assert_eq!(
            super_workflow::compare(&left, operator, right),
            expected,
            "{left} {} {right}",
            operator.name()
        );
    }
}

#[test]
fn a_group_of_any_holds_when_one_does() {
    use crate::types::{Condition, Operator};
    let frame = frame_with(
        &[
            ("a", serde_json::json!({"status": 503})),
            ("b", serde_json::json!({"status": 200})),
        ],
        &[],
    );
    let compare = |id: &str| Condition::Compare {
        left: format!("{{{{steps.{id}.status}}}}"),
        operator: Operator::Equal,
        right: Some("200".into()),
    };
    assert!(
        super_workflow::holds(&Condition::Any(vec![compare("a"), compare("b")]), &frame).unwrap()
    );
    assert!(
        !super_workflow::holds(&Condition::All(vec![compare("a"), compare("b")]), &frame).unwrap()
    );
    let numeric = Condition::Compare {
        left: "{{steps.a.status}}".into(),
        operator: Operator::Greater,
        right: Some("499".into()),
    };
    assert!(super_workflow::holds(&numeric, &frame).unwrap());
}
