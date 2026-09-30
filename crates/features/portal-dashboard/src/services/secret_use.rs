use portal_feature::{Action, ApiError, Area, Right, Rights};
use portal_widget::WidgetInstance;
use serde_json::Value;

use crate::types::EditedLayout;

pub const READ_SECRETS: Right = Right::new(Area::Secrets, Action::Read);
pub const SECRET: &str = "secret";
pub const URL: &str = "url";

fn secret_of(settings: &Value) -> Option<(&str, Option<&str>)> {
    settings
        .get(SECRET)
        .and_then(Value::as_str)
        .filter(|name| !name.trim().is_empty())
        .map(|name| (name, settings.get(URL).and_then(Value::as_str)))
}

pub fn secrets_allowed(
    edited: &EditedLayout,
    stored: &[WidgetInstance],
    rights: &Rights,
) -> Result<(), ApiError> {
    if rights.allows(READ_SECRETS) {
        return Ok(());
    }
    let kept: Vec<(&str, Option<&str>)> = stored
        .iter()
        .filter_map(|widget| secret_of(&widget.settings))
        .collect();
    let borrowed = edited.widgets.iter().enumerate().find(|(_, widget)| {
        secret_of(&widget.instance.settings).is_some_and(|pair| !kept.contains(&pair))
    });
    match borrowed {
        Some((index, _)) => Err(ApiError::Forbidden(format!(
            "widgets[{index}].secret names a secret, which needs {READ_SECRETS}"
        ))),
        None => Ok(()),
    }
}
