use std::sync::Arc;

use portal_config::{ConfigStore, Revision, Revisioned};
use portal_feature::{ApiError, Rights};
use portal_widget::WidgetInstance;

use super::change_layout::dashboard_file;
use crate::repositories::{remove_library_entry, write_library_entry};
use crate::services::{check_entry, library_views, needs_allowed, secrets_allowed};
use crate::types::{EditedLayout, EditedWidget, NeedsOf, no_needs};

#[derive(Clone)]
pub struct LibraryCases {
    configuration: Arc<ConfigStore>,
    needs_of: NeedsOf,
}

impl LibraryCases {
    pub const UNKNOWN: &'static str = "no such widget in the library";
    pub const TAKEN: &'static str = "is used by another widget of the library";

    pub fn new(configuration: Arc<ConfigStore>) -> LibraryCases {
        LibraryCases {
            configuration,
            needs_of: no_needs(),
        }
    }

    pub fn with_needs(self, needs_of: NeedsOf) -> LibraryCases {
        LibraryCases { needs_of, ..self }
    }

    pub fn list(&self) -> Result<Revisioned<Vec<(WidgetInstance, usize)>>, ApiError> {
        let snapshot = self.configuration.read();
        let views = library_views(&snapshot.document)
            .map_err(|message| ApiError::Internal(format!("dashboard section: {message}")))?;
        Ok(Revisioned::new(views, snapshot.revision))
    }

    pub async fn save(
        &self,
        existing: Option<&str>,
        entry: WidgetInstance,
        (revision, rights): (&Revision, &Rights),
    ) -> Result<Revisioned<(WidgetInstance, usize)>, ApiError> {
        let errors = check_entry(&entry);
        if !errors.is_empty() {
            return Err(ApiError::Invalid(errors));
        }
        let stored: Vec<WidgetInstance> = self
            .list()?
            .value
            .into_iter()
            .map(|(widget, _)| widget)
            .collect();
        let mut entry = entry;
        match existing {
            Some(id) => {
                if !stored.iter().any(|widget| widget.id.as_deref() == Some(id)) {
                    return Err(ApiError::NotFound(Self::UNKNOWN));
                }
                entry.id = Some(id.to_string());
            }
            None => {
                if let Some(id) = &entry.id
                    && stored.iter().any(|widget| widget.id.as_ref() == Some(id))
                {
                    return Err(ApiError::invalid("id", Self::TAKEN));
                }
            }
        }
        let edited = EditedLayout {
            sections: Vec::new(),
            widgets: vec![EditedWidget {
                key: None,
                instance: entry.clone(),
            }],
        };
        secrets_allowed(&edited, &stored, rights)?;
        needs_allowed(&edited, &stored, rights, &self.needs_of)?;
        let target = dashboard_file(&self.configuration)?;
        let mut saved = None;
        let (_, snapshot) = self
            .configuration
            .update(&target, revision, |document| {
                saved = Some(write_library_entry(document, existing, &entry));
                Ok(())
            })
            .await
            .map_err(|error| match error {
                ApiError::Invalid(errors) => ApiError::Invalid(
                    errors
                        .into_iter()
                        .map(|error| renamed(error, saved.as_deref()))
                        .collect(),
                ),
                other => other,
            })?;
        let views = library_views(&snapshot.document).map_err(ApiError::Internal)?;
        let found = views
            .into_iter()
            .find(|(widget, _)| widget.id.is_some() && widget.id.as_deref() == saved.as_deref())
            .ok_or(ApiError::NotFound(Self::UNKNOWN))?;
        Ok(Revisioned::new(found, snapshot.revision))
    }

    pub async fn delete(&self, id: &str, revision: &Revision) -> Result<Revision, ApiError> {
        let listed = self.list()?.value;
        let Some((_, placed)) = listed
            .iter()
            .find(|(widget, _)| widget.id.as_deref() == Some(id))
        else {
            return Err(ApiError::NotFound(Self::UNKNOWN));
        };
        if *placed > 0 {
            return Err(ApiError::Conflict(format!(
                "{id} is placed on the home page {placed} {}; remove it from the layout first",
                if *placed == 1 { "time" } else { "times" }
            )));
        }
        let target = dashboard_file(&self.configuration)?;
        let (_, snapshot) = self
            .configuration
            .update(&target, revision, |document| {
                remove_library_entry(document, id);
                Ok(())
            })
            .await?;
        Ok(snapshot.revision)
    }
}

fn renamed(error: portal_feature::FieldError, id: Option<&str>) -> portal_feature::FieldError {
    let field = error.field.clone();
    let Some(id) = id else {
        return error;
    };
    match field.strip_prefix(&format!("dashboard.library.{id}.")) {
        Some(rest) => portal_feature::FieldError::new(rest, error.message),
        None => error,
    }
}
