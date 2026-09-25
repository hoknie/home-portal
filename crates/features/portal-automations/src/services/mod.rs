mod cache;
mod dispatch;
mod journal;
mod matcher;
mod schedule_book;
mod scripts;
mod system_clock;
mod validation;
mod webhook_book;
mod webhook_checks;

#[cfg(test)]
pub mod tests;

pub use cache::AutomationCache;
pub use dispatch::{AutomationSink, FinishGuard, StatusRelay, execute};
pub use journal::Journal;
pub use matcher::matching;
pub use schedule_book::ScheduleBook;
pub use scripts::ScriptsDirectory;
pub use system_clock::SystemClock;
pub use validation::{decoded, decoded_webhooks, validate_automations};
pub use webhook_book::WebhookBook;
pub use webhook_checks::webhook_placeholder_errors;
