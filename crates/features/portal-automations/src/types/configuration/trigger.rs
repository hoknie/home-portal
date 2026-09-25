use portal_feature::{EventName, FieldError};
use portal_model::ServiceState;

use super::{CronFilter, Filters};
use crate::parsers::parse_cron;
use crate::types::FilterName;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trigger {
    pub event: EventName,
    pub filters: Filters,
}

impl Trigger {
    pub const EVENT_KEY: &'static str = "event";
    pub const PREFIX: &'static str = "when.";
    pub const NEVER_FIRES: &'static str = "never fires: no date in the next four years matches";

    pub fn decode(table: &toml::Table) -> Result<Trigger, Vec<FieldError>> {
        let named = table.get(Self::EVENT_KEY).and_then(toml::Value::as_str);
        let event = named.map(EventName::from).unwrap_or(EventName::Unknown);
        if event == EventName::Unknown {
            return Err(vec![FieldError::new(
                "when.event",
                format!(
                    "must be one of {}",
                    crate::types::Catalogue::event_names().join(", ")
                ),
            )]);
        }
        let allowed = FilterName::for_event(event);
        let mut filters = Filters::default();
        let mut errors = Vec::new();
        for (key, value) in table.iter().filter(|(key, _)| *key != Self::EVENT_KEY) {
            let field = format!("{}{key}", Self::PREFIX);
            match FilterName::of(key).filter(|filter| allowed.contains(filter)) {
                None => errors.push(FieldError::new(field, Self::foreign(event, allowed))),
                Some(filter) => {
                    if let Err(message) = Self::apply(&mut filters, filter, value) {
                        errors.push(FieldError::new(field, message));
                    }
                }
            }
        }
        if event == EventName::Schedule && filters.cron.is_none() && errors.is_empty() {
            errors.push(FieldError::new("when.cron", "is required for a schedule"));
        }
        if filters
            .states
            .from
            .iter()
            .any(|state| state == ServiceState::Unknown.name())
            && !filters.states.from_unknown
        {
            errors.push(FieldError::new(
                "when.from",
                "names unknown, which matches only with from_unknown = true",
            ));
        }
        if errors.is_empty() {
            Ok(Trigger { event, filters })
        } else {
            Err(errors)
        }
    }

    fn foreign(event: EventName, allowed: &[FilterName]) -> String {
        if allowed.is_empty() {
            return format!("is not a filter of {}, which has none", event.name());
        }
        let names: Vec<&str> = allowed.iter().map(|filter| filter.key()).collect();
        format!(
            "is not a filter of {}; its filters are {}",
            event.name(),
            names.join(", ")
        )
    }

    fn apply(filters: &mut Filters, filter: FilterName, value: &toml::Value) -> Result<(), String> {
        match filter {
            FilterName::FromUnknown => {
                filters.states.from_unknown = value.as_bool().ok_or("must be true or false")?;
            }
            FilterName::Cron => {
                let expression = value.as_str().ok_or("must be a cron expression")?;
                let schedule = parse_cron(expression)?;
                if schedule
                    .next_after(jiff::Timestamp::now(), &jiff::tz::TimeZone::UTC)
                    .is_none()
                {
                    return Err(Self::NEVER_FIRES.to_string());
                }
                filters.cron = Some(CronFilter {
                    expression: expression.to_string(),
                    schedule,
                });
            }
            FilterName::Services => filters.services = Self::names(value)?,
            FilterName::Users => filters.users = Self::names(value)?,
            FilterName::Environments => filters.environments = Self::names(value)?,
            FilterName::From => filters.states.from = Self::states(value)?,
            FilterName::To => filters.states.to = Self::states(value)?,
            FilterName::Webhooks => filters.webhooks = Self::names(value)?,
        }
        Ok(())
    }

    fn names(value: &toml::Value) -> Result<Vec<String>, String> {
        let list = value.as_array().ok_or("must be a list of names")?;
        list.iter()
            .map(|item| {
                item.as_str()
                    .map(str::to_string)
                    .ok_or_else(|| "must be a list of names".to_string())
            })
            .collect()
    }

    fn states(value: &toml::Value) -> Result<Vec<String>, String> {
        let names = Self::names(value)?;
        let states = crate::types::Catalogue::states();
        match names.iter().find(|name| !states.contains(&name.as_str())) {
            Some(unknown) => Err(format!(
                "names {unknown}, which is not a state; the states are {}",
                states.join(", ")
            )),
            None => Ok(names),
        }
    }
}
