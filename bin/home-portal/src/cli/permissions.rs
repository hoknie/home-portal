use std::io::{IsTerminal, Write};
use std::process::ExitCode;
use std::sync::Arc;

use clap::builder::styling::Style;
use portal_config::{ConfigStore, configuration_path};
use portal_permissions::{
    Advice, OwnerKind, PermissionState, PermissionView, PermissionsFeature, PermissionsView,
};

use super::failure::fail;
use super::palette::{ALERT, EMPHASIS, ERROR, MUTED, SUCCESS, WARNING};

pub const CODE_WIDTH: usize = 28;
pub const STATE_WIDTH: usize = 14;

pub async fn permissions() -> ExitCode {
    let store = match configuration_path().and_then(|location| ConfigStore::open_located(&location))
    {
        Ok(store) => Arc::new(store),
        Err(error) => return fail(error),
    };
    let feature = PermissionsFeature::new(store);
    if let Err(error) = feature.request_permissions().run().await {
        return fail(format!("cannot ask for the permissions: {error:?}"));
    }
    let view = feature.show_permissions().run();
    let text = if std::io::stdout().is_terminal() {
        described(&view)
    } else {
        plain(&view)
    };
    let _ = write!(anstream::stdout(), "{text}");
    status_of(&view)
}

pub fn status_of(view: &PermissionsView) -> ExitCode {
    if view.denied() {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

pub fn plain(view: &PermissionsView) -> String {
    view.permissions
        .iter()
        .map(|permission| format!("{}\t{}\n", permission.code.code(), permission.state.name()))
        .collect()
}

pub fn described(view: &PermissionsView) -> String {
    let owner = owner_of(view);
    let mut lines = vec![format!(
        "{MUTED}permissions of{MUTED:#} {EMPHASIS}{owner}{EMPHASIS:#}"
    )];
    for permission in &view.permissions {
        let style = state_style(permission.state);
        let state = permission.state.name();
        let mut line = format!(
            "{:<CODE_WIDTH$} {style}{state:<STATE_WIDTH$}{style:#}",
            permission.code.code()
        );
        if let Some(advice) = advice_of(permission, &owner) {
            line.push_str(&format!(" {advice}"));
        }
        lines.push(line.trim_end().to_string());
    }
    lines.push(String::new());
    lines.join("\n")
}

fn owner_of(view: &PermissionsView) -> String {
    match view.owner.kind {
        OwnerKind::Terminal => format!("{} (this process runs in it)", view.owner.name),
        OwnerKind::Binary => view.owner.name.clone(),
    }
}

fn advice_of(permission: &PermissionView, owner: &str) -> Option<String> {
    if permission.state.settled() {
        return None;
    }
    let advice = permission.advice?;
    Some(match advice {
        Advice::AllowInSettings | Advice::AnswerThePrompt | Advice::GrantFullDiskAccess => {
            format!(
                "{}: {} for {owner}",
                advice.text(),
                permission.pane().title()
            )
        }
        Advice::ConnectAVolume | Advice::ApplicationNotFound | Advice::CheckFailed => {
            advice.text().to_string()
        }
    })
}

fn state_style(state: PermissionState) -> Style {
    match state {
        PermissionState::Granted => SUCCESS,
        PermissionState::Denied => ERROR,
        PermissionState::Pending => WARNING,
        PermissionState::Unknown => ALERT,
        PermissionState::NotApplicable => MUTED,
    }
}
