use super::{placeholders_of, render, unknown_placeholders};

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

#[test]
fn the_last_line_drops_colour_codes() {
    assert_eq!(
        super::last_line("\u{1b}[32mok\u{1b}[0m\n\u{1b}[1;31mcontainer not found\u{1b}[m\n"),
        Some("container not found".to_string())
    );
    assert_eq!(
        super::last_line("\u{1b}]0;title\u{7}text"),
        Some("text".to_string())
    );
}

#[test]
fn the_last_line_of_a_progress_meter_is_its_last_state() {
    assert_eq!(
        super::last_line("  0%\r 45%\r100% done\r\n"),
        Some("100% done".to_string())
    );
    assert_eq!(super::last_line("abcdef\rxy"), Some("xycdef".to_string()));
}

#[test]
fn trailing_blank_lines_are_skipped_and_nothing_gives_no_line() {
    assert_eq!(
        super::last_line("first\nlast one\n\n   \n"),
        Some("last one".to_string())
    );
    assert_eq!(super::last_line("\n\n"), None);
    assert_eq!(super::last_line(""), None);
}

#[test]
fn a_body_path_may_name_indexes_and_keys_of_any_case() {
    assert_eq!(
        placeholders_of("{{webhook.body.commits.0.headCommit}} {{webhook.body}}"),
        vec!["webhook.body.commits.0.headCommit", "webhook.body"]
    );
    assert!(placeholders_of("{{webhook.Body}}").is_empty());
    assert!(placeholders_of("{{webhook.body.}}").is_empty());
    assert_eq!(
        unknown_placeholders("{{webhook.body.a.1}}", &["webhook.body"]),
        Vec::<String>::new()
    );
    assert_eq!(
        unknown_placeholders("{{webhook.body.a}}", &["webhook.id"]),
        vec!["webhook.body.a".to_string()]
    );
}

#[test]
fn event_values_render_the_body_as_text_and_resolve_a_whole_placeholder_as_a_value() {
    let event = crate::types::EventValues::new(
        vec![("webhook.id".to_string(), "x".to_string())],
        Some(serde_json::json!({"zones": [1, 3], "name": "gate"})),
    );
    assert_eq!(
        event.render("{{webhook.body.name}} {{webhook.body.zones}} {{webhook.body.none}}!"),
        "gate [1,3] !"
    );
    assert_eq!(
        event.resolve(" {{webhook.body.zones}} "),
        serde_json::json!([1, 3])
    );
    assert_eq!(event.resolve("{{webhook.id}}"), serde_json::json!("x"));
    assert_eq!(
        event.resolve("zones: {{webhook.body.zones}}"),
        serde_json::json!("zones: [1,3]")
    );
}
