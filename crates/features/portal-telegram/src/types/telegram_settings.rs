use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct TelegramSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub secret: Option<String>,
    #[serde(default)]
    pub chat_id: Option<String>,
    #[serde(default = "TelegramSettings::default_states")]
    pub states: Vec<String>,
    #[serde(default = "TelegramSettings::yes")]
    pub recovered: bool,
}

impl TelegramSettings {
    pub const RECOVERED_STATE: &'static str = "up";

    pub fn default_states() -> Vec<String> {
        vec!["down".to_string(), "unreadable".to_string()]
    }

    pub fn yes() -> bool {
        true
    }

    pub fn announces(&self, state: &str, from_unknown: bool) -> bool {
        if !self.enabled {
            return false;
        }
        if state == Self::RECOVERED_STATE {
            return self.recovered && !from_unknown;
        }
        self.states.iter().any(|wanted| wanted == state)
    }
}

impl Default for TelegramSettings {
    fn default() -> TelegramSettings {
        TelegramSettings {
            enabled: false,
            secret: None,
            chat_id: None,
            states: Self::default_states(),
            recovered: true,
        }
    }
}
