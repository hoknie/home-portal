#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Section {
    Services,
    Dashboard,
    Users,
    Notifications,
    Proxy,
    Dns,
    Automations,
    Webhooks,
    Secrets,
    Workflows,
}

impl Section {
    pub const ALL: [Section; 10] = [
        Section::Services,
        Section::Dashboard,
        Section::Users,
        Section::Notifications,
        Section::Proxy,
        Section::Dns,
        Section::Automations,
        Section::Webhooks,
        Section::Secrets,
        Section::Workflows,
    ];

    pub const MAIN_KEYS: [&'static str; 9] = [
        "network",
        "environments",
        "modules",
        "storage",
        "interface",
        "scripts",
        "files",
        "include",
        "configuration",
    ];

    pub const WORKFLOW_KEYS: [&'static str; 8] = [
        "id",
        "title",
        "enabled",
        "description",
        "tags",
        "timeout_seconds",
        "inputs",
        "steps",
    ];

    pub fn key(self) -> &'static str {
        match self {
            Section::Services => "services",
            Section::Dashboard => "dashboard",
            Section::Users => "users",
            Section::Notifications => "notifications",
            Section::Proxy => "proxy",
            Section::Dns => "dns",
            Section::Automations => "automations",
            Section::Webhooks => "webhooks",
            Section::Secrets => "secrets",
            Section::Workflows => "workflows",
        }
    }

    pub fn tables(self) -> &'static [&'static str] {
        match self {
            Section::Automations => &["automations", "automation_settings"],
            Section::Services => &["services"],
            Section::Dashboard => &["dashboard"],
            Section::Users => &["users"],
            Section::Notifications => &["notifications"],
            Section::Proxy => &["proxy"],
            Section::Dns => &["dns"],
            Section::Webhooks => &["webhooks"],
            Section::Secrets => &["secrets"],
            Section::Workflows => &["workflows"],
        }
    }

    pub fn default_home(self) -> &'static str {
        match self {
            Section::Services => "services.toml",
            Section::Dashboard => "dashboard.toml",
            Section::Users => "users.toml",
            Section::Notifications => "notifications.toml",
            Section::Proxy => "proxy.toml",
            Section::Dns => "dns.toml",
            Section::Automations => "automations.toml",
            Section::Webhooks => "webhooks.toml",
            Section::Secrets => "secrets.toml",
            Section::Workflows => "workflows/",
        }
    }

    pub fn is_folder(self) -> bool {
        self == Section::Workflows
    }

    pub fn of_table(table: &str) -> Option<Section> {
        Section::ALL
            .into_iter()
            .find(|section| section.tables().contains(&table))
    }

    pub fn of_key(key: &str) -> Option<Section> {
        Section::ALL
            .into_iter()
            .find(|section| section.key() == key)
    }
}
