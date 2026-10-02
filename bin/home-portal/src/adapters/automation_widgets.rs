use std::collections::BTreeMap;
use std::time::Duration;

use async_trait::async_trait;
use portal_automations::{SourceCall, ValueType, WidgetSupport, WidgetTemplates as Templates};
use portal_feature::{ApiError, FieldError};
use portal_widgets::{
    DeclaredPath, NameKind, Started, TemplateContext, WidgetReferences, WidgetRuns, WidgetSource,
    WidgetTemplates,
};
use serde_json::Value;

pub struct AutomationWidgets {
    pub support: WidgetSupport,
}

struct Context(Templates);

impl TemplateContext for Context {
    fn text(&self, template: &str) -> Result<String, String> {
        self.0.text(template)
    }

    fn value(&self, template: &str) -> Result<Value, String> {
        self.0.value(template)
    }

    fn set_item(&mut self, item: Option<(Value, usize)>) {
        self.0.set_item(item);
    }
}

pub fn value_type(kind: NameKind) -> ValueType {
    match kind {
        NameKind::Text => ValueType::Text,
        NameKind::Number => ValueType::Number,
        NameKind::Any => ValueType::Any,
    }
}

pub fn source_call(source: &WidgetSource) -> SourceCall {
    match source {
        WidgetSource::Workflow { id, inputs } => SourceCall::Workflow {
            id: id.clone(),
            inputs: inputs.clone(),
        },
        WidgetSource::Script {
            script,
            args,
            timeout,
        } => SourceCall::Script {
            script: script.clone(),
            args: args.clone(),
            timeout: *timeout,
        },
    }
}

impl WidgetTemplates for AutomationWidgets {
    fn problem(
        &self,
        template: &str,
        allows: &dyn Fn(&str) -> Result<NameKind, String>,
    ) -> Option<String> {
        self.support
            .check_template
            .run(template, &|name| allows(name).map(value_type))
    }

    fn open(
        &self,
        data: &Value,
        id: &str,
        title: Option<&str>,
        fetched_at: Option<String>,
    ) -> Box<dyn TemplateContext> {
        Box::new(Context(
            self.support.open_templates.run(data, id, title, fetched_at),
        ))
    }
}

impl WidgetReferences for AutomationWidgets {
    fn call_problems(
        &self,
        workflow: &str,
        inputs: &BTreeMap<String, Option<Value>>,
    ) -> Vec<FieldError> {
        self.support.check_call.run(workflow, inputs)
    }

    fn script_problem(&self, script: &str) -> Option<String> {
        self.support.check_script.run(script)
    }

    fn event_fields(&self, automation: &str) -> Option<(String, Vec<String>)> {
        self.support.event_fields.run(automation)
    }

    fn declared_paths(&self, workflow: &str) -> Vec<DeclaredPath> {
        self.support
            .declared_paths
            .run(workflow)
            .into_iter()
            .map(|path| DeclaredPath {
                path: path.path,
                description: path.description,
                kind: path.kind.to_string(),
            })
            .collect()
    }

    fn source_timeout(&self, workflow: &str) -> Duration {
        self.support.source_timeout.run(workflow)
    }
}

#[async_trait]
impl WidgetRuns for AutomationWidgets {
    async fn run_source(
        &self,
        widget: &str,
        title: &str,
        source: &WidgetSource,
    ) -> Result<Value, String> {
        self.support
            .run_source
            .run(widget, title, &source_call(source))
            .await
    }

    fn start_automation(
        &self,
        started: Started<'_>,
        automation: &str,
        fields: &[(String, String)],
    ) -> Result<u64, ApiError> {
        self.support.start_automation.run(
            started.widget,
            started.title,
            automation,
            fields,
            started.by,
        )
    }

    fn start_workflow(
        &self,
        started: Started<'_>,
        workflow: &str,
        inputs: Vec<(String, Value)>,
    ) -> Result<u64, ApiError> {
        self.support
            .start_workflow
            .run(started.widget, started.title, workflow, inputs, started.by)
    }
}
