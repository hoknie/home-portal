mod automations;
mod catalogue;
mod receive;
mod runs;
mod webhooks;

#[cfg(test)]
mod tests;

pub use automations::{UNKNOWN_AUTOMATION, create, list, remove, update};
pub use catalogue::{catalogue, schedule, scripts};
pub use receive::{UNKNOWN_WEBHOOK, receive};
pub use runs::{run_now, runs};
pub use webhooks::{
    create_webhook, delete_webhook, issue_token, list_webhooks, remove_token, update_webhook,
};
