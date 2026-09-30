use super::Action;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Area {
    Services,
    Layout,
    Network,
    Modules,
    Scripts,
    Secrets,
    HostPermissions,
    Portal,
    Proxy,
    Dns,
    Automations,
    Webhooks,
    Users,
    Workflows,
    Notifications,
}

const READ_UPDATE: &[Action] = &[Action::Read, Action::Update];
const WITH_EXECUTE: &[Action] = &[
    Action::Read,
    Action::Create,
    Action::Update,
    Action::Delete,
    Action::Execute,
];
const CRUD: &[Action] = &[Action::Read, Action::Create, Action::Update, Action::Delete];

impl Area {
    pub const ALL: [Area; 15] = [
        Area::Services,
        Area::Layout,
        Area::Network,
        Area::Modules,
        Area::Scripts,
        Area::Secrets,
        Area::HostPermissions,
        Area::Portal,
        Area::Proxy,
        Area::Dns,
        Area::Automations,
        Area::Webhooks,
        Area::Users,
        Area::Workflows,
        Area::Notifications,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Area::Services => "services",
            Area::Layout => "layout",
            Area::Network => "network",
            Area::Modules => "modules",
            Area::Scripts => "scripts",
            Area::Secrets => "secrets",
            Area::HostPermissions => "host-permissions",
            Area::Portal => "portal",
            Area::Proxy => "proxy",
            Area::Dns => "dns",
            Area::Automations => "automations",
            Area::Webhooks => "webhooks",
            Area::Users => "users",
            Area::Workflows => "workflows",
            Area::Notifications => "notifications",
        }
    }

    pub fn from_name(name: &str) -> Option<Area> {
        Area::ALL.into_iter().find(|area| area.name() == name)
    }

    pub fn actions(self) -> &'static [Action] {
        match self {
            Area::Proxy | Area::Dns | Area::Notifications => READ_UPDATE,
            Area::Network | Area::HostPermissions | Area::Modules => READ_UPDATE,
            Area::Automations | Area::Webhooks | Area::Workflows => WITH_EXECUTE,
            Area::Users | Area::Scripts => CRUD,
            Area::Services => &[Action::Create, Action::Update, Action::Delete],
            Area::Layout | Area::Portal => &[Action::Update],
            Area::Secrets => &[Action::Read],
        }
    }

    pub fn accepts(self, action: Action) -> bool {
        self.actions().contains(&action)
    }
}
