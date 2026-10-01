use portal_config::deserialize_section;
use portal_feature::FieldError;
use portal_model::ServiceState;
use serde::Deserialize;
use toml_edit::{DocumentMut, Item};

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
    pub const RULE_KEYS: [&'static str; 2] = ["states", "recovered"];

    pub fn default_states() -> Vec<String> {
        vec!["down".to_string(), "unreadable".to_string()]
    }

    pub fn read(document: &DocumentMut) -> Result<Rules, String> {
        let section: Section = deserialize_section(document)?;
        let notifications = section.notifications.unwrap_or_default();
        Ok(Rules {
            states: notifications.states.unwrap_or_else(Self::default_states),
            recovered: notifications.recovered.unwrap_or(true),
        })
    }

    pub fn problems(document: &DocumentMut) -> Vec<FieldError> {
        let rules = match Self::read(document) {
            Ok(rules) => rules,
            Err(message) => return vec![FieldError::new(Self::SECTION, message)],
        };
        let mut problems = Self::misplaced(document);
        problems.extend(
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
                }),
        );
        problems
    }

    fn misplaced(document: &DocumentMut) -> Vec<FieldError> {
        let Some(section) = document.get(Self::SECTION).and_then(Item::as_table_like) else {
            return Vec::new();
        };
        section
            .iter()
            .filter_map(|(channel, item)| item.as_table_like().map(|table| (channel, table)))
            .flat_map(|(channel, table)| {
                Self::RULE_KEYS
                    .into_iter()
                    .filter(move |key| table.contains_key(key))
                    .map(move |key| {
                        FieldError::new(
                            format!("{}.{channel}.{key}", Self::SECTION),
                            format!("is no longer read; the rules live in {}", Self::SECTION),
                        )
                    })
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
