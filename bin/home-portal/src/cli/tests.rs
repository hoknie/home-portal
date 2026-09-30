use std::process::ExitCode;

use portal_permissions::{
    Advice, Owner, PermissionCode, PermissionState, PermissionView, PermissionsView,
};

use super::failure::failure_line;
use super::palette::examples;
use super::permissions::{described, plain, status_of};

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

fn view(states: &[(PermissionCode, PermissionState, Option<Advice>)]) -> PermissionsView {
    PermissionsView {
        platform: "macos",
        owner: Owner::detect(Some("Apple_Terminal".to_string()), None),
        permissions: states
            .iter()
            .map(|(code, state, advice)| PermissionView {
                code: code.clone(),
                state: *state,
                advice: *advice,
                learned_at: None,
            })
            .collect(),
    }
}

#[test]
fn permissions_through_a_pipe_are_plain_code_and_state_lines() {
    let shown = view(&[
        (PermissionCode::LocalNetwork, PermissionState::Granted, None),
        (
            PermissionCode::Automation("System Events".to_string()),
            PermissionState::Pending,
            Some(Advice::AnswerThePrompt),
        ),
    ]);
    assert_eq!(
        plain(&shown),
        "local-network\tgranted\nautomation:System Events\tpending\n"
    );
    assert_eq!(status_of(&shown), ExitCode::SUCCESS);
}

#[test]
fn a_denied_permission_names_its_pane_and_owner_and_fails() {
    let shown = view(&[(
        PermissionCode::LocalNetwork,
        PermissionState::Denied,
        Some(Advice::AllowInSettings),
    )]);
    let text = anstream::adapter::strip_str(&described(&shown)).to_string();
    assert!(text.contains("permissions of Terminal"), "{text}");
    assert!(
        text.contains("Privacy & Security > Local Network for Terminal"),
        "{text}"
    );
    assert_eq!(status_of(&shown), ExitCode::FAILURE);
}
