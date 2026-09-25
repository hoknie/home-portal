#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Storage {
    Sessions,
    Icons,
    History,
    Automations,
    Scripts,
    Caddy,
}

impl Storage {
    pub const ALL: [Storage; 6] = [
        Storage::Sessions,
        Storage::Icons,
        Storage::History,
        Storage::Automations,
        Storage::Scripts,
        Storage::Caddy,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Storage::Sessions => "sessions",
            Storage::Icons => "icons",
            Storage::History => "history",
            Storage::Automations => "automations",
            Storage::Scripts => "scripts",
            Storage::Caddy => "caddy",
        }
    }

    pub fn default_name(self) -> &'static str {
        match self {
            Storage::Sessions => "sessions.json",
            other => other.key(),
        }
    }
}
