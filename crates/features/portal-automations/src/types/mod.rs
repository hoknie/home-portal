mod automations_state;
mod catalogue;
mod configuration;
mod journal;
mod running;
mod schedule;
mod stored;
mod webhooks;
mod wildcards;

#[cfg(test)]
mod tests;

pub use automations_state::AutomationsState;
pub use catalogue::{Catalogue, Choice, FilterName};
pub use configuration::{
    Automation, AutomationsSection, CronFilter, Filters, RawAutomation, RawRun, RunSettings,
    StateFilter, Trigger,
};
pub use journal::{Outcome, RunFilter, RunRecord, Seen, SkipReason};
pub use running::{
    ActiveRun, Admission, Finished, Invocation, Pending, Refusal, RefusalCode, RunControl,
    ScriptEntry, StopAnswer, Tail,
};
pub use schedule::Schedule;
pub use stored::StoredRun;
pub use webhooks::{RawMarks, RawWebhook, Reception, Webhook, WebhookAction};
pub use wildcards::Wildcards;
