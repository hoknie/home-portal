use time::OffsetDateTime;

use crate::types::{Finding, Owner, PermissionCode, PermissionState, PermissionView};

pub fn logged(view: &PermissionView, owner: &Owner) {
    let code = view.code.code();
    if view.state.settled() {
        tracing::info!(permission = %code, state = view.state.name(), "permission");
        return;
    }
    let advice = view.advice.map_or("", |advice| advice.text());
    tracing::warn!(
        permission = %code,
        state = view.state.name(),
        pane = view.pane().title(),
        owner = %owner.name,
        advice,
        "permission is not granted"
    );
}

pub fn view_of(code: PermissionCode, found: Option<(Finding, OffsetDateTime)>) -> PermissionView {
    match found {
        Some((finding, learned_at)) => PermissionView {
            code,
            state: finding.state,
            advice: finding.advice,
            learned_at: Some(learned_at),
        },
        None => PermissionView {
            code,
            state: PermissionState::Unknown,
            advice: None,
            learned_at: None,
        },
    }
}
