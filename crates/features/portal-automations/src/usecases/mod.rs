mod automations;
mod webhooks;
mod widget_runs;
mod workflows;

pub use automations::{ChangeAutomation, CreateAutomation, DeleteAutomation, ListAutomations};
pub use webhooks::{
    ChangeWebhook, CreateWebhook, DeleteWebhook, IssueToken, ListWebhooks, RemoveToken, RunWebhook,
};
pub use widget_runs::{
    CheckWidgetCall, CheckWidgetScript, CheckWidgetTemplate, OpenWidgetTemplates,
    ReadDeclaredPaths, ReadEventFields, ReadSourceTimeout, RunWidgetSource, StartWidgetAutomation,
    StartWidgetWorkflow,
};
pub use workflows::{
    ChangeWorkflow, CreateWorkflow, DeleteWorkflow, ListWorkflows, ReadPortalValues, RunWorkflow,
    TransformValue, WorkflowCatalogue,
};
