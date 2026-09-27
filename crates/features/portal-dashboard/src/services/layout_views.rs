use portal_feature::ApiError;
use portal_model::Environment;
use toml_edit::DocumentMut;

use super::layout;
use crate::types::LayoutView;

pub fn layout_view(
    document: &DocumentMut,
    filter: Option<&Environment>,
) -> Result<LayoutView, ApiError> {
    let layout = layout(document)
        .map_err(|message| ApiError::Internal(format!("dashboard section: {message}")))?;
    let widgets = layout
        .widgets
        .into_iter()
        .enumerate()
        .filter(|(_, widget)| filter.is_none_or(|environment| widget.visible_to(environment)))
        .collect();
    Ok(LayoutView {
        sections: layout.sections,
        widgets,
    })
}
