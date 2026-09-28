use portal_config::deserialize_section;
use portal_feature::FieldError;
use portal_model::ServiceState;
use serde::Deserialize;
use toml_edit::DocumentMut;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rules {
    pub states: Vec<String>,
    pub recovered: bool,
}

#[derive(Debug, Default, Deserialize)]
struct Section {
    #[serde(default)]
    notifications: Option<Notifications>,
}

#[derive(Debug, Default, Deserialize)]
struct Notifications {
    #[serde(default)]
    states: Option<Vec<String>>,
    #[serde(default)]
    recovered: Option<bool>,
    #[serde(default)]
    telegram: Option<Legacy>,
}

#[derive(Debug, Default, Deserialize)]
struct Legacy {
    #[serde(default)]
    states: Option<Vec<String>>,
    #[serde(default)]
    recovered: Option<bool>,
}

impl Default for Rules {
    fn default() -> Rules {
        Rules {
            states: Self::default_states(),
            recovered: true,
        }
    }
}

impl Rules {
    pub const SECTION: &'static str = "notifications";
    pub const RECOVERED_STATE: &'static str = "up";
    pub const LEGACY: &'static str = "telegram";

    pub fn default_states() -> Vec<String> {
        vec!["down".to_string(), "unreadable".to_string()]
    }

    pub fn read(document: &DocumentMut) -> Result<Rules, String> {
        let section: Section = deserialize_section(document)?;
        let notifications = section.notifications.unwrap_or_default();
        let legacy = notifications.telegram.unwrap_or_default();
        Ok(Rules {
            states: notifications
                .states
                .or(legacy.states)
                .unwrap_or_else(Self::default_states),
            recovered: notifications.recovered.or(legacy.recovered).unwrap_or(true),
        })
    }

    pub fn problems(document: &DocumentMut) -> Vec<FieldError> {
        let rules = match Self::read(document) {
            Ok(rules) => rules,
            Err(message) => return vec![FieldError::new(Self::SECTION, message)],
        };
        rules
            .states
            .iter()
            .filter(|state| {
                !ServiceState::ALL
                    .iter()
                    .any(|known| known.name() == state.as_str())
            })
            .map(|state| {
                FieldError::new(
                    format!("{}.states", Self::SECTION),
                    format!("names {state}, which is not a service state"),
                )
            })
            .collect()
    }

    pub fn announces(&self, state: &str, from_unknown: bool) -> bool {
        if state == Self::RECOVERED_STATE {
            return self.recovered && !from_unknown;
        }
        self.states.iter().any(|wanted| wanted == state)
    }
}
