use super::failure::failure_line;
use super::palette::examples;

#[test]
fn a_failure_is_one_error_line_once_styles_are_stripped() {
    let line = failure_line("the password must not be empty");
    assert_eq!(
        anstream::adapter::strip_str(&line).to_string(),
        "error: the password must not be empty"
    );
}

#[test]
fn examples_are_aligned_on_the_widest_example() {
    let text = examples("Examples", &[("a", "first"), ("abc", "second")]).to_string();
    assert_eq!(text, "Examples:\n  a    first\n  abc  second");
}
