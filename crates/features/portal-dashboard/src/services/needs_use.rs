use portal_feature::{ApiError, NeedScope, Rights, WidgetNeed};
use portal_widget::WidgetInstance;

use crate::types::{EditedLayout, NeedsOf};

pub fn needs_allowed(
    edited: &EditedLayout,
    stored: &[WidgetInstance],
    rights: &Rights,
    needs_of: &NeedsOf,
) -> Result<(), ApiError> {
    let kept: Vec<(String, WidgetNeed)> = stored
        .iter()
        .flat_map(|widget| {
            let id = widget.id.clone().unwrap_or_default();
            needs_of(&widget.kind, &widget.settings)
                .into_iter()
                .map(move |need| (id.clone(), need))
        })
        .collect();
    for (index, widget) in edited.widgets.iter().enumerate() {
        for need in needs_of(&widget.instance.kind, &widget.instance.settings) {
            if rights.allows(need.right) {
                continue;
            }
            let unchanged = kept.iter().any(|(key, old)| {
                old.right == need.right
                    && old.fingerprint == need.fingerprint
                    && match need.scope {
                        NeedScope::SameWidget => {
                            widget.instance.id.as_deref() == Some(key.as_str())
                        }
                        NeedScope::AnyWidget => true,
                    }
            });
            if !unchanged {
                return Err(ApiError::Forbidden(format!(
                    "widgets[{index}].{} needs {}, which you do not hold; a widget that already had it may keep it unchanged",
                    need.field, need.right
                )));
            }
        }
    }
    Ok(())
}
