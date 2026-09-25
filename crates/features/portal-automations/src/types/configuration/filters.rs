use super::{CronFilter, StateFilter};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filters {
    pub services: Vec<String>,
    pub states: StateFilter,
    pub users: Vec<String>,
    pub environments: Vec<String>,
    pub webhooks: Vec<String>,
    pub cron: Option<CronFilter>,
}
