use std::time::Instant;

use portal_feature::{ApiError, EventName, EventSink, PortalEvent};
use time::OffsetDateTime;

use super::{AutomationSink, WebhookBook};
use crate::types::{Webhook, WebhookAction};

pub fn received(
    sink: &AutomationSink,
    book: &WebhookBook,
    webhook: &Webhook,
    variables: &[(String, String)],
    (client, by): (&str, Option<String>),
) -> Result<Option<u64>, ApiError> {
    book.allow(&webhook.id, Instant::now())
        .map_err(|retry_after_seconds| ApiError::TooManyRequests {
            retry_after_seconds,
        })?;
    let event = PortalEvent::of(
        EventName::WebhookReceived,
        OffsetDateTime::now_utc(),
        &[
            ("webhook.id", webhook.id.as_str()),
            ("webhook.title", webhook.title.as_str()),
            ("client.address", client),
        ],
    )
    .with_variables(variables);
    Ok(match webhook.action {
        WebhookAction::Event => {
            sink.emit(event);
            None
        }
        WebhookAction::Script(_) | WebhookAction::Workflow(_) => {
            sink.run_webhook_by(webhook, event, by)
        }
    })
}
