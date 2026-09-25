#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventName {
    Schedule,
    PortalStarted,
    PortalStopping,
    ServiceStatusChanged,
    ServiceCreated,
    ServiceUpdated,
    ServiceDeleted,
    UserSignedIn,
    UserSignedOut,
    UserSignInFailed,
    ConfigurationChanged,
    WebhookReceived,
    Manual,
    Unknown,
}

impl EventName {
    pub const ALL: [EventName; 13] = [
        EventName::Schedule,
        EventName::PortalStarted,
        EventName::PortalStopping,
        EventName::ServiceStatusChanged,
        EventName::ServiceCreated,
        EventName::ServiceUpdated,
        EventName::ServiceDeleted,
        EventName::UserSignedIn,
        EventName::UserSignedOut,
        EventName::UserSignInFailed,
        EventName::ConfigurationChanged,
        EventName::WebhookReceived,
        EventName::Manual,
    ];

    const PORTAL: &'static [&'static str] = &["portal.address", "portal.version"];
    const VISITOR: &'static [&'static str] = &["user.name", "client.address", "client.environment"];

    pub fn name(&self) -> &'static str {
        match self {
            EventName::Schedule => "schedule",
            EventName::PortalStarted => "portal.started",
            EventName::PortalStopping => "portal.stopping",
            EventName::ServiceStatusChanged => "service.status-changed",
            EventName::ServiceCreated => "service.created",
            EventName::ServiceUpdated => "service.updated",
            EventName::ServiceDeleted => "service.deleted",
            EventName::UserSignedIn => "user.signed-in",
            EventName::UserSignedOut => "user.signed-out",
            EventName::UserSignInFailed => "user.sign-in-failed",
            EventName::ConfigurationChanged => "configuration.changed",
            EventName::WebhookReceived => "webhook.received",
            EventName::Manual => "manual",
            EventName::Unknown => "unknown",
        }
    }

    pub fn fields(&self) -> &'static [&'static str] {
        match self {
            EventName::Schedule => &["schedule.cron", "schedule.at"],
            EventName::PortalStarted | EventName::PortalStopping => Self::PORTAL,
            EventName::ServiceStatusChanged => &[
                "service.id",
                "service.name",
                "status.from",
                "status.to",
                "status.error",
                "status.diagnosis",
            ],
            EventName::ServiceCreated | EventName::ServiceDeleted => {
                &["service.id", "service.name", "user.name"]
            }
            EventName::ServiceUpdated => &[
                "service.id",
                "service.name",
                "service.previous_id",
                "user.name",
            ],
            EventName::UserSignedIn | EventName::UserSignedOut => Self::VISITOR,
            EventName::UserSignInFailed => &[
                "user.name",
                "client.address",
                "client.environment",
                "sign_in.reason",
            ],
            EventName::ConfigurationChanged => {
                &["configuration.revision", "configuration.previous_revision"]
            }
            EventName::WebhookReceived => &["webhook.id", "webhook.title", "client.address"],
            EventName::Manual | EventName::Unknown => &[],
        }
    }
}

impl From<&str> for EventName {
    fn from(name: &str) -> EventName {
        EventName::ALL
            .into_iter()
            .find(|candidate| candidate.name() == name)
            .unwrap_or(EventName::Unknown)
    }
}
