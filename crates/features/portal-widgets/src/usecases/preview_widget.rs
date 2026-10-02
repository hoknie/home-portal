use std::sync::Arc;

use portal_feature::{ApiError, Principal};
use serde_json::Value;
use time::format_description::well_known::Rfc3339;

use crate::providers::{CustomWidgetProvider, source_need};
use crate::responses::{DeclaredPathResponse, PreviewErrorResponse, WidgetPreviewResponse};
use crate::services::{WidgetMeta, decode_custom, render_widget};
use crate::types::{CustomWidget, WidgetSource};

pub struct PreviewAsked {
    pub id: Option<String>,
    pub settings: Value,
    pub run: bool,
    pub sample: Option<Value>,
}

#[derive(Clone)]
pub struct PreviewWidget {
    widgets: Arc<CustomWidgetProvider>,
}

impl PreviewWidget {
    pub const LARGEST_SAMPLE: usize = 16 * 1024;

    pub fn new(widgets: Arc<CustomWidgetProvider>) -> PreviewWidget {
        PreviewWidget { widgets }
    }

    pub async fn run(
        &self,
        asked: PreviewAsked,
        principal: &Principal,
    ) -> Result<WidgetPreviewResponse, ApiError> {
        if let Some(sample) = &asked.sample
            && sample.to_string().len() > Self::LARGEST_SAMPLE
        {
            return Err(ApiError::BadRequest(format!(
                "sample holds more than {} KiB",
                Self::LARGEST_SAMPLE / 1024
            )));
        }
        let decoded = decode_custom(&asked.settings, self.widgets.context());
        let id = asked.id.clone().unwrap_or_else(|| "preview".to_string());
        let saved = asked.id.as_deref().and_then(|id| self.widgets.instance(id));
        let title = saved.as_ref().and_then(|instance| instance.title.clone());
        let mut ran = false;
        let mut problem = None;
        let mut fetched_at = None;
        let data = match (&decoded, asked.run) {
            (
                Ok(CustomWidget {
                    source: Some(source),
                    ..
                }),
                true,
            ) => {
                if let Some((right, _)) = asked.settings.get("source").and_then(source_need)
                    && !principal.rights.allows(right)
                {
                    return Err(ApiError::Forbidden(format!(
                        "running this source needs {right}"
                    )));
                }
                let run = self
                    .widgets
                    .ports
                    .runs
                    .run_source(&id, title.as_deref().unwrap_or(&id), source)
                    .await;
                ran = true;
                match run {
                    Ok(data) => data,
                    Err(failure) => {
                        problem = Some(failure);
                        Value::Null
                    }
                }
            }
            _ => {
                let same_source = saved.as_ref().is_some_and(|instance| {
                    instance.settings.get("source") == asked.settings.get("source")
                });
                match asked
                    .id
                    .as_deref()
                    .and_then(|id| self.widgets.memory.remembered(id))
                    .filter(|_| same_source)
                {
                    Some(kept) => {
                        fetched_at = kept.at.format(&Rfc3339).ok();
                        kept.data
                    }
                    None => asked.sample.clone().unwrap_or(Value::Null),
                }
            }
        };
        let (blocks, errors) = match &decoded {
            Ok(widget) => {
                let meta = WidgetMeta {
                    id: &id,
                    title: title.as_deref(),
                    fetched_at,
                };
                match render_widget(self.widgets.ports.templates.as_ref(), widget, &data, &meta) {
                    Ok(blocks) => (blocks, Vec::new()),
                    Err(failure) => {
                        problem.get_or_insert(failure);
                        (Vec::new(), Vec::new())
                    }
                }
            }
            Err(errors) => (
                Vec::new(),
                errors
                    .iter()
                    .cloned()
                    .map(PreviewErrorResponse::of)
                    .collect(),
            ),
        };
        let paths = match &decoded {
            Ok(CustomWidget {
                source: Some(WidgetSource::Workflow { id, .. }),
                ..
            }) => self
                .widgets
                .ports
                .references
                .declared_paths(id)
                .into_iter()
                .map(DeclaredPathResponse::of)
                .collect(),
            _ => Vec::new(),
        };
        Ok(WidgetPreviewResponse {
            blocks,
            errors,
            data,
            ran,
            problem,
            paths,
        })
    }
}
