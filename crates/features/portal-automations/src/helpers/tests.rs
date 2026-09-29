use super::{render, unknown_placeholders};

fn lookup(name: &str) -> Option<&'static str> {
    match name {
        "service.id" => Some("nas"),
        "status.error" => Some("{{service.name}}"),
        _ => None,
    }
}

#[test]
fn a_known_placeholder_is_replaced_and_other_text_stays_as_written() {
    let cases = [
        ("{{service.id}}", "nas"),
        ("--id={{service.id}}!", "--id=nas!"),
        ("{{ nope", "{{ nope"),
        ("{x}", "{x}"),
        ("{{}}", "{{}}"),
        ("{{{service.id}}}", "{nas}"),
        ("{{Service.Id}}", "{{Service.Id}}"),
        ("{{service.name}}", "{{service.name}}"),
        ("{{service.id}}{{service.id}}", "nasnas"),
    ];
    for (template, expected) in cases {
        assert_eq!(render(template, lookup), expected, "{template}");
    }
}

#[test]
fn a_value_is_never_expanded_a_second_time() {
    assert_eq!(render("{{status.error}}", lookup), "{{service.name}}");
}

#[test]
fn an_unknown_field_is_reported_and_a_known_one_is_not() {
    let allowed = ["service.id"];
    assert_eq!(
        unknown_placeholders("{{service.id}} {{service.name}} {{ no }}", &allowed),
        vec!["service.name"]
    );
    assert!(unknown_placeholders("plain", &allowed).is_empty());
}

#[test]
fn a_placeholder_may_hold_digits_after_the_first_letter() {
    let values = |name: &str| (name == "webhook.build2").then_some("7");
    assert_eq!(render("{{webhook.build2}}", values), "7");
    assert_eq!(render("{{webhook.2build}}", values), "{{webhook.2build}}");
}

#[test]
fn a_shape_keeps_keys_and_shortens_lists_texts_and_depth() {
    use serde_json::json;
    let deep = (0..12).fold(json!("bottom"), |inner, _| json!({ "in": inner }));
    let value = json!({"list": [1, 2, 3, 4, 5], "text": "y".repeat(500), "deep": deep});
    let shape: serde_json::Value = serde_json::from_str(&super::shape_of(&value)).unwrap();
    assert_eq!(shape["list"], json!([1, 2, 3]));
    assert_eq!(shape["text"].as_str().unwrap().len(), 200);
    assert_eq!(
        shape.pointer("/deep/in/in/in/in/in/in/in"),
        Some(&json!({}))
    );
    let huge = json!((0..5).map(|index| json!({ "k": index, "v": "z".repeat(200), "more": (0..3).map(|_| "w".repeat(200)).collect::<Vec<_>>() })).collect::<Vec<_>>());
    let wide = json!(
        (0..100)
            .map(|index| (format!("key{index}"), huge.clone()))
            .collect::<serde_json::Map<_, _>>()
    );
    assert!(super::shape_of(&wide).len() <= super::LARGEST_SHAPE);
}
