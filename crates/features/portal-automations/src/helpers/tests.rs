use super::{render, script_shape_problem, unknown_placeholders};

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
fn a_script_path_must_stay_inside_the_directory() {
    assert!(script_shape_problem("restart.sh").is_none());
    assert!(script_shape_problem("media/restart.sh").is_none());
    assert!(script_shape_problem("./restart.sh").is_none());
    for refused in ["", "  ", "/bin/sh", "../home-portal.toml", "media/../../x"] {
        assert!(script_shape_problem(refused).is_some(), "{refused:?}");
    }
}

#[test]
fn a_placeholder_may_hold_digits_after_the_first_letter() {
    let values = |name: &str| (name == "webhook.build2").then_some("7");
    assert_eq!(render("{{webhook.build2}}", values), "7");
    assert_eq!(render("{{webhook.2build}}", values), "{{webhook.2build}}");
}
