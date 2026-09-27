use crate::types::{ActiveRun, Automation, RunRecord};

#[derive(Debug, Clone)]
pub struct AutomationView {
    pub automation: Automation,
    pub last: Option<RunRecord>,
    pub active: Option<ActiveRun>,
}
