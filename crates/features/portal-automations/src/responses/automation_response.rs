use serde::Serialize;

use super::{MarksResponse, RunResponse, RunSettingsResponse, WhenResponse};
use crate::types::{ActiveRun, Automation, RunRecord};

#[derive(Debug, Clone, Serialize)]
pub struct AutomationResponse {
    pub id: String,
    pub title: String,
    #[serde(flatten)]
    pub marks: MarksResponse,
    pub cooldown_seconds: u64,
    pub when: WhenResponse,
    pub run: RunSettingsResponse,
    pub last_run: Option<RunResponse>,
    pub active_run: Option<RunResponse>,
}

impl AutomationResponse {
    pub fn of(
        automation: &Automation,
        last_run: Option<&RunRecord>,
        active_run: Option<&ActiveRun>,
    ) -> AutomationResponse {
        AutomationResponse {
            id: automation.id.clone(),
            title: automation.title.clone(),
            marks: MarksResponse {
                enabled: automation.enabled,
                tags: automation.tags.clone(),
            },
            cooldown_seconds: automation.cooldown_seconds,
            when: WhenResponse::of(&automation.trigger),
            run: RunSettingsResponse::of(&automation.run),
            last_run: last_run.map(RunResponse::of),
            active_run: active_run.map(RunResponse::active),
        }
    }
}
