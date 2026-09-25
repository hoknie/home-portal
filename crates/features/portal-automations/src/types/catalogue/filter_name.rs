use portal_feature::EventName;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterName {
    Services,
    From,
    To,
    FromUnknown,
    Users,
    Environments,
    Webhooks,
    Cron,
}

impl FilterName {
    pub const ALL: [FilterName; 8] = [
        FilterName::Services,
        FilterName::From,
        FilterName::To,
        FilterName::FromUnknown,
        FilterName::Users,
        FilterName::Environments,
        FilterName::Webhooks,
        FilterName::Cron,
    ];

    pub fn key(self) -> &'static str {
        match self {
            FilterName::Services => "services",
            FilterName::From => "from",
            FilterName::To => "to",
            FilterName::FromUnknown => "from_unknown",
            FilterName::Users => "users",
            FilterName::Environments => "environments",
            FilterName::Webhooks => "webhooks",
            FilterName::Cron => "cron",
        }
    }

    pub fn of(key: &str) -> Option<FilterName> {
        Self::ALL.into_iter().find(|filter| filter.key() == key)
    }

    pub fn for_event(event: EventName) -> &'static [FilterName] {
        match event {
            EventName::Schedule => &[FilterName::Cron],
            EventName::ServiceStatusChanged => &[
                FilterName::Services,
                FilterName::From,
                FilterName::To,
                FilterName::FromUnknown,
            ],
            EventName::ServiceCreated | EventName::ServiceUpdated | EventName::ServiceDeleted => {
                &[FilterName::Services]
            }
            EventName::UserSignedIn | EventName::UserSignedOut | EventName::UserSignInFailed => {
                &[FilterName::Users, FilterName::Environments]
            }
            EventName::WebhookReceived => &[FilterName::Webhooks],
            EventName::PortalStarted
            | EventName::PortalStopping
            | EventName::ConfigurationChanged
            | EventName::Manual
            | EventName::Unknown => &[],
        }
    }
}
