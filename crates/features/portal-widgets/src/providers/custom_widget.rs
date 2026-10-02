use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use portal_config::ConfigStore;
use portal_feature::{
    FieldError, Module, ModuleSwitches, WidgetLimits, WidgetNeed, WidgetProblem, WidgetProvider,
};
use portal_widget::{WidgetInstance, WidgetsSection};
use serde_json::Value;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use super::widget_memory::{Remembered, WidgetMemory};
use super::widget_needs::needs_of;
use crate::responses::CustomWidgetData;
use crate::services::{WidgetMeta, decode_custom, render_widget};
use crate::types::{CustomWidget, WidgetPorts, WidgetSource};

pub struct CustomWidgetProvider {
    pub configuration: Arc<ConfigStore>,
    pub ports: WidgetPorts,
    pub memory: Arc<WidgetMemory>,
}

impl CustomWidgetProvider {
    pub const INVALID: &'static str = "the widget's settings are not valid";

    pub fn context(&self) -> &WidgetPorts {
        &self.ports
    }

    pub fn automations_on(&self) -> bool {
        ModuleSwitches::resolve(&self.configuration.read().document)
            .unwrap_or_default()
            .is_on(Module::Automations)
    }

    pub fn instance(&self, id: &str) -> Option<WidgetInstance> {
        WidgetsSection::read(&self.configuration.read().document)
            .unwrap_or_default()
            .into_iter()
            .find(|instance| {
                instance.kind == CustomWidget::KIND && instance.id.as_deref() == Some(id)
            })
    }

    pub fn source_key(source: Option<&WidgetSource>) -> String {
        format!("{source:?}")
    }

    pub async fn fresh_data(
        &self,
        id: &str,
        title: &str,
        widget: &CustomWidget,
        force: bool,
    ) -> Result<Remembered, String> {
        let key = Self::source_key(widget.source.as_ref());
        let now = OffsetDateTime::now_utc();
        let kept = self.memory.remembered(id).filter(|kept| kept.source == key);
        if let Some(kept) = &kept
            && !force
            && (now - kept.at).whole_seconds()
                < i64::try_from(widget.refresh.as_secs()).unwrap_or(i64::MAX)
        {
            return Ok(kept.clone());
        }
        let Some(source) = &widget.source else {
            let empty = Remembered {
                data: Value::Null,
                at: now,
                source: key,
            };
            self.memory.remember(id, empty.clone());
            return Ok(empty);
        };
        let data = self.ports.runs.run_source(id, title, source).await?;
        let remembered = Remembered {
            data,
            at: OffsetDateTime::now_utc(),
            source: key,
        };
        self.memory.remember(id, remembered.clone());
        Ok(remembered)
    }

    pub fn rendered(
        &self,
        id: &str,
        title: Option<&str>,
        widget: &CustomWidget,
        remembered: &Remembered,
    ) -> Result<Value, String> {
        let meta = WidgetMeta {
            id,
            title,
            fetched_at: remembered.at.format(&Rfc3339).ok(),
        };
        let blocks = render_widget(
            self.ports.templates.as_ref(),
            widget,
            &remembered.data,
            &meta,
        )?;
        serde_json::to_value(CustomWidgetData { blocks }).map_err(|problem| problem.to_string())
    }
}

#[async_trait]
impl WidgetProvider for CustomWidgetProvider {
    fn kind(&self) -> &'static str {
        CustomWidget::KIND
    }

    fn refresh(&self) -> Duration {
        Duration::from_secs(CustomWidget::DEFAULT_REFRESH)
    }

    fn check(&self, settings: &Value) -> Vec<FieldError> {
        decode_custom(settings, self.context())
            .err()
            .unwrap_or_default()
    }

    async fn data(&self, _settings: &Value) -> Result<Value, WidgetProblem> {
        Err(WidgetProblem::new(Self::INVALID))
    }

    async fn data_of(&self, id: &str, settings: &Value) -> Result<Value, WidgetProblem> {
        let widget = decode_custom(settings, self.context())
            .map_err(|_| WidgetProblem::new(Self::INVALID))?;
        let title = self.instance(id).and_then(|instance| instance.title);
        let force = self.memory.take_rerun(id);
        let remembered = self
            .fresh_data(id, title.as_deref().unwrap_or(id), &widget, force)
            .await
            .map_err(WidgetProblem::new)?;
        let rendered = self
            .rendered(id, title.as_deref(), &widget, &remembered)
            .map_err(WidgetProblem::new)?;
        self.memory.rendered(id, settings.to_string());
        Ok(rendered)
    }

    fn limits(&self, settings: &Value) -> WidgetLimits {
        let Ok(widget) = decode_custom(settings, self.context()) else {
            return WidgetLimits::of(self.refresh());
        };
        let timeout = match &widget.source {
            None => WidgetLimits::LONGEST_WAIT,
            Some(WidgetSource::Script { timeout, .. }) => *timeout + Duration::from_secs(5),
            Some(WidgetSource::Workflow { id, .. }) => {
                self.ports.references.source_timeout(id) + Duration::from_secs(5)
            }
        };
        WidgetLimits {
            refresh: widget.refresh,
            timeout,
        }
    }

    fn needs(&self, settings: &Value) -> Vec<WidgetNeed> {
        needs_of(settings)
    }

    fn expired(&self, id: &str) -> bool {
        if self.memory.rerun_asked(id) {
            return true;
        }
        let current = self
            .instance(id)
            .map(|instance| instance.settings.to_string());
        current.is_some() && current != self.memory.rendered_with(id)
    }

    fn available(&self) -> bool {
        self.automations_on()
    }

    fn public_data(&self, data: Value) -> Value {
        match serde_json::from_value::<CustomWidgetData>(data) {
            Ok(widget) => serde_json::to_value(widget.without_buttons()).unwrap_or(Value::Null),
            Err(_) => Value::Null,
        }
    }
}
