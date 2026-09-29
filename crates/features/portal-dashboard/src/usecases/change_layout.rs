use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;

use portal_config::{ConfigStore, Revision, Revisioned, Section};
use portal_feature::ApiError;

use crate::repositories::write_layout;
use crate::services::{check_edited, layout_view, renamed_for_the_editor};
use crate::types::{EditedLayout, LayoutView};

#[derive(Clone)]
pub struct ChangeLayout {
    configuration: Arc<ConfigStore>,
}

impl ChangeLayout {
    pub const WIDGETS: &'static str = "dashboard.widgets";
    pub const SECTIONS: &'static str = "dashboard.sections";
    pub const DASHBOARD: &'static str = "dashboard";

    pub fn new(configuration: Arc<ConfigStore>) -> ChangeLayout {
        ChangeLayout { configuration }
    }

    pub async fn run(
        &self,
        edited: &EditedLayout,
        revision: &Revision,
    ) -> Result<Revisioned<LayoutView>, ApiError> {
        let errors = check_edited(edited);
        if !errors.is_empty() {
            return Err(ApiError::Invalid(errors));
        }
        let target = self.target_file()?;
        let written = self
            .configuration
            .update(&target, revision, |document| {
                write_layout(document, edited);
                Ok(())
            })
            .await;
        let (_, snapshot) = written.map_err(|error| match error {
            ApiError::Invalid(errors) => ApiError::Invalid(
                errors
                    .into_iter()
                    .map(|error| renamed_for_the_editor(error, edited))
                    .collect(),
            ),
            other => other,
        })?;
        let view = layout_view(&snapshot.document, None)?;
        Ok(Revisioned::new(view, snapshot.revision))
    }

    fn target_file(&self) -> Result<PathBuf, ApiError> {
        let snapshot = self.configuration.read();
        let origins = &snapshot.origins;
        let files: BTreeSet<PathBuf> = [Self::WIDGETS, Self::SECTIONS]
            .into_iter()
            .flat_map(|section| {
                (0..origins.count(section)).filter_map(move |index| origins.of(section, index))
            })
            .map(PathBuf::from)
            .collect();
        if files.len() > 1 {
            let names: Vec<String> = files
                .iter()
                .map(|path| path.display().to_string())
                .collect();
            return Err(ApiError::Conflict(format!(
                "the layout is spread over {}; move every [[dashboard.widgets]] and [[dashboard.sections]] entry into one file to edit it here",
                names.join(" and ")
            )));
        }
        Ok(files
            .into_iter()
            .next()
            .or_else(|| origins.table(Self::DASHBOARD).map(PathBuf::from))
            .unwrap_or_else(|| self.configuration.home_of(Section::Dashboard)))
    }
}
