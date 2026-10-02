use crate::usecases::{
    CheckWidgetCall, CheckWidgetScript, CheckWidgetTemplate, OpenWidgetTemplates,
    ReadDeclaredPaths, ReadEventFields, ReadSourceTimeout, RunWidgetSource, StartWidgetAutomation,
    StartWidgetWorkflow,
};

#[derive(Clone)]
pub struct WidgetSupport {
    pub check_template: CheckWidgetTemplate,
    pub open_templates: OpenWidgetTemplates,
    pub check_call: CheckWidgetCall,
    pub check_script: CheckWidgetScript,
    pub event_fields: ReadEventFields,
    pub declared_paths: ReadDeclaredPaths,
    pub source_timeout: ReadSourceTimeout,
    pub run_source: RunWidgetSource,
    pub start_automation: StartWidgetAutomation,
    pub start_workflow: StartWidgetWorkflow,
}
