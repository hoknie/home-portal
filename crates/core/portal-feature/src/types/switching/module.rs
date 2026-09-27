#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Module {
    Proxy,
    Dns,
    Automations,
    Webhooks,
    Users,
}

impl Module {
    pub const ALL: [Module; 5] = [
        Module::Proxy,
        Module::Dns,
        Module::Automations,
        Module::Webhooks,
        Module::Users,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Module::Proxy => "proxy",
            Module::Dns => "dns",
            Module::Automations => "automations",
            Module::Webhooks => "webhooks",
            Module::Users => "users",
        }
    }

    pub fn from_name(name: &str) -> Option<Module> {
        Module::ALL.into_iter().find(|module| module.name() == name)
    }

    pub fn requires(self) -> &'static [Module] {
        match self {
            Module::Dns => &[Module::Proxy],
            Module::Webhooks => &[Module::Automations],
            Module::Proxy | Module::Automations | Module::Users => &[],
        }
    }

    pub fn default_on(self) -> bool {
        matches!(self, Module::Automations | Module::Webhooks)
    }

    pub fn legacy_section(self) -> Option<&'static str> {
        match self {
            Module::Proxy => Some("proxy"),
            Module::Dns => Some("dns"),
            Module::Automations | Module::Webhooks | Module::Users => None,
        }
    }

    pub fn index(self) -> usize {
        match self {
            Module::Proxy => 0,
            Module::Dns => 1,
            Module::Automations => 2,
            Module::Webhooks => 3,
            Module::Users => 4,
        }
    }
}
