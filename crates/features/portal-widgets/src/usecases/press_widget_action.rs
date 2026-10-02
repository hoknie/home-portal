use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use portal_feature::{ApiError, Principal, Right};
use portal_model::Environment;
use serde_json::Value;

use crate::ports::Started;
use crate::providers::{CustomWidgetProvider, RUN_AUTOMATIONS, RUN_WORKFLOWS, source_need};
use crate::services::{WidgetMeta, decode_custom, widget_context};
use crate::types::WidgetTarget;

#[derive(Clone)]
pub struct PressWidgetAction {
    widgets: Arc<CustomWidgetProvider>,
    pressed: Arc<Mutex<HashMap<String, Instant>>>,
}

impl PressWidgetAction {
    pub const EVERY: Duration = Duration::from_secs(2);
    pub const UNKNOWN: &'static str = "no such widget action";
    pub const AUTOMATIONS_OFF: &'static str = "the automations module is off";

    pub fn new(widgets: Arc<CustomWidgetProvider>) -> PressWidgetAction {
        PressWidgetAction {
            widgets,
            pressed: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn run(
        &self,
        (widget_id, action_id): (&str, &str),
        environment: &Environment,
        principal: &Principal,
    ) -> Result<Option<u64>, ApiError> {
        let instance = self
            .widgets
            .instance(widget_id)
            .filter(|instance| instance.visible_to(environment))
            .ok_or(ApiError::NotFound(Self::UNKNOWN))?;
        let widget = decode_custom(&instance.settings, self.widgets.context())
            .map_err(|_| ApiError::NotFound(Self::UNKNOWN))?;
        let action = widget
            .action(action_id)
            .ok_or(ApiError::NotFound(Self::UNKNOWN))?
            .clone();
        if !self.widgets.automations_on() {
            return Err(ApiError::Conflict(Self::AUTOMATIONS_OFF.to_string()));
        }
        let needed: Option<Right> = match &action.target {
            WidgetTarget::Automation { .. } => Some(RUN_AUTOMATIONS),
            WidgetTarget::Workflow { .. } => Some(RUN_WORKFLOWS),
            WidgetTarget::Refresh => instance
                .settings
                .get("source")
                .and_then(source_need)
                .map(|(right, _)| right),
            WidgetTarget::Link(_) => return Err(ApiError::NotFound(Self::UNKNOWN)),
        };
        if let Some(right) = needed
            && !principal.rights.allows(right)
        {
            return Err(ApiError::Forbidden(format!("this button needs {right}")));
        }
        self.throttle(&format!("{widget_id}/{action_id}"))?;
        let remembered = self.widgets.memory.remembered(widget_id);
        let data = remembered
            .as_ref()
            .map_or(Value::Null, |kept| kept.data.clone());
        let title = instance.title.clone();
        let frame = widget_context(
            self.widgets.ports.templates.as_ref(),
            &data,
            &WidgetMeta {
                id: widget_id,
                title: title.as_deref(),
                fetched_at: None,
            },
        );
        let started = Started {
            widget: widget_id,
            title: title.as_deref(),
            by: &principal.name,
        };
        let runs = &self.widgets.ports.runs;
        match &action.target {
            WidgetTarget::Automation { id, fields } => {
                let mut rendered = Vec::with_capacity(fields.len());
                for (name, template) in fields {
                    rendered.push((
                        name.clone(),
                        frame.text(template).map_err(ApiError::Conflict)?,
                    ));
                }
                runs.start_automation(started, id, &rendered).map(Some)
            }
            WidgetTarget::Workflow { id, inputs } => {
                let mut given = Vec::with_capacity(inputs.len());
                for (name, value) in inputs {
                    let value = match value {
                        Value::String(template) => {
                            frame.value(template).map_err(ApiError::Conflict)?
                        }
                        literal => literal.clone(),
                    };
                    given.push((name.clone(), value));
                }
                runs.start_workflow(started, id, given).map(Some)
            }
            WidgetTarget::Refresh => {
                self.widgets.memory.ask_rerun(widget_id);
                Ok(None)
            }
            WidgetTarget::Link(_) => Err(ApiError::NotFound(Self::UNKNOWN)),
        }
    }

    fn throttle(&self, key: &str) -> Result<(), ApiError> {
        let mut pressed = self.pressed.lock().unwrap_or_else(PoisonError::into_inner);
        let now = Instant::now();
        if let Some(last) = pressed.get(key)
            && now.duration_since(*last) < Self::EVERY
        {
            let left = Self::EVERY.saturating_sub(now.duration_since(*last));
            return Err(ApiError::TooManyRequests {
                retry_after_seconds: left.as_secs().max(1),
            });
        }
        pressed.insert(key.to_string(), now);
        Ok(())
    }
}
