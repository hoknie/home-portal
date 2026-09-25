use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use portal_config::ConfigStore;
use portal_feature::{FieldError, WidgetProblem, WidgetProvider};
use serde_json::{Value, json};
use time::OffsetDateTime;
use url::Url;

use crate::clients::FeedClient;
use crate::services::{expand, offset_of, parse_feed};
use crate::types::{CalendarEvent, CalendarSettings};

pub struct CalendarProvider {
    client: FeedClient,
    configuration: Arc<ConfigStore>,
}

impl CalendarProvider {
    pub const KIND: &'static str = "calendar";
    pub const REFRESH: Duration = Duration::from_secs(15 * 60);
    pub const SHORTEST_REFRESH: Duration = Duration::from_secs(5 * 60);

    pub fn new(configuration: Arc<ConfigStore>) -> Result<CalendarProvider, String> {
        Ok(CalendarProvider {
            client: FeedClient::new()?,
            configuration,
        })
    }

    fn settings(settings: &Value) -> Result<CalendarSettings, String> {
        serde_json::from_value(settings.clone()).map_err(|error| error.to_string())
    }

    pub async fn events(
        &self,
        settings: &CalendarSettings,
        now: OffsetDateTime,
    ) -> Result<Vec<CalendarEvent>, WidgetProblem> {
        let offset = offset_of(settings.timezone.as_deref()).map_err(WidgetProblem::new)?;
        let url =
            Url::parse(&settings.url).map_err(|error| WidgetProblem::new(error.to_string()))?;
        let secret = settings
            .secret
            .as_deref()
            .and_then(|name| self.configuration.secret(name));
        let text = self.client.fetch(&url, secret.as_ref()).await?;
        let feed = parse_feed(&text, offset).map_err(WidgetProblem::new)?;
        let until = now + time::Duration::days(i64::from(settings.days));
        let mut events: Vec<CalendarEvent> = feed
            .events
            .iter()
            .flat_map(|event| expand(event, feed.name.as_deref(), now, until))
            .collect();
        events.sort_by_key(|event| event.start);
        events.truncate(usize::from(settings.limit));
        Ok(events)
    }
}

#[async_trait]
impl WidgetProvider for CalendarProvider {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn refresh(&self) -> Duration {
        Self::REFRESH.max(Self::SHORTEST_REFRESH)
    }

    fn check(&self, settings: &Value) -> Vec<FieldError> {
        let settings = match Self::settings(settings) {
            Ok(settings) => settings,
            Err(message) => return vec![FieldError::new("settings", message)],
        };
        let mut errors = Vec::new();
        match Url::parse(&settings.url) {
            Ok(url) if matches!(url.scheme(), "http" | "https") => {}
            _ => errors.push(FieldError::new(
                "url",
                "must be an absolute http or https URL",
            )),
        }
        if !CalendarSettings::DAYS.contains(&settings.days) {
            errors.push(FieldError::new("days", "must be between 1 and 31"));
        }
        if !CalendarSettings::LIMIT.contains(&settings.limit) {
            errors.push(FieldError::new("limit", "must be between 1 and 50"));
        }
        if let Err(message) = offset_of(settings.timezone.as_deref()) {
            errors.push(FieldError::new("timezone", message));
        }
        if let Some(name) = &settings.secret
            && self.configuration.secret(name).is_none()
        {
            errors.push(FieldError::new(
                "secret",
                format!("names {name}, which is not set"),
            ));
        }
        errors
    }

    async fn data(&self, settings: &Value) -> Result<Value, WidgetProblem> {
        let settings = Self::settings(settings).map_err(WidgetProblem::new)?;
        let events = self.events(&settings, OffsetDateTime::now_utc()).await?;
        Ok(json!({ "events": events }))
    }
}
