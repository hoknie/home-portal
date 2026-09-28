mod automations_state;
mod catalogue;
mod configuration;
mod inputs;
mod journal;
mod progress;
mod running;
mod schedule;
mod stepping;
mod stored;
mod transforming;
mod views;
mod webhooks;
mod wildcards;
mod workflow;

#[cfg(test)]
mod tests;

pub use automations_state::{AutomationsState, WorkflowCases};
pub use catalogue::{Catalogue, Choice, FilterName};
pub use configuration::{
    Automation, AutomationsSection, CronFilter, Filters, RawAutomation, RawRun, RunSettings,
    StateFilter, Trigger, WorkflowCall,
};
pub use inputs::{InputDeclaration, InputType, InputValue, RawInput};
pub use journal::{Outcome, RunFilter, RunRecord, Seen, SkipReason};
pub use progress::{
    Ending, Flow, HttpAnswer, HttpRequest, Place, ProbeResult, StatusResult, StepOutcome,
    StepReport, Trace, TraceEntry,
};
pub use running::{
    ActiveRun, Admission, Finished, Invocation, Pending, Refusal, RefusalCode, RunControl,
    ScriptEntry, StopAnswer, Tail,
};
pub use schedule::Schedule;
pub use stepping::{FieldDescription, KINDS, KindDescription, METHODS, OUTCOMES, kind_named};
pub use stored::StoredRun;
pub use transforming::{
    ArgumentDescription, ArgumentType, DEEPEST_EACH, FILTERS, FilterCall, FilterDescription,
    MOST_OPERATIONS, OPERATIONS, Operation, Placeholder, RawOperation, ValueType, filter_named,
    result_type,
};
pub use views::{
    AutomationView, CreatedWebhook, WebhookView, WorkflowCatalogueView, WorkflowUsage, WorkflowView,
};
pub use webhooks::{RawMarks, RawWebhook, Reception, Webhook, WebhookAction};
pub use wildcards::Wildcards;
pub use workflow::{
    Condition, HttpStep, LoopMode, Operator, RawCondition, RawStep, RawWorkflow, SetValue, Step,
    StepKind, Workflow,
};
