use portal_feature::Module;
use time::OffsetDateTime;

use super::AutomationSink;
use crate::helpers::manual_event;
use crate::ports::AutomationStarter;
use crate::types::{Outcome, Workflow};

pub const AUTOMATIONS_OFF: &str = "the automations module is off";
pub const MOST_CHAINED: usize = Workflow::DEEPEST_CALLS;

impl AutomationStarter for AutomationSink {
    fn start(
        &self,
        automation: &str,
        fields: &[(String, String)],
        origin: &[String],
    ) -> Result<u64, String> {
        if !self.cache.switches().is_on(Module::Automations) {
            return Err(AUTOMATIONS_OFF.to_string());
        }
        let Some(found) = self.cache.find(automation) else {
            return Err(format!("no automation {automation}"));
        };
        if !found.enabled {
            return Err(format!("the automation {automation} is disabled"));
        }
        if origin.iter().any(|id| id == automation) {
            return Err(format!(
                "the automation {automation} already started this run"
            ));
        }
        if origin.len() > MOST_CHAINED {
            return Err(format!(
                "automations start each other more than {MOST_CHAINED} deep"
            ));
        }
        let mut event = manual_event(&found, OffsetDateTime::now_utc(), &[]);
        for (name, value) in fields {
            if let Some(field) = event.fields.iter_mut().find(|(key, _)| key == name) {
                field.1 = value.clone();
            }
        }
        Ok(self.admit(&found, event, (None, origin.to_vec())))
    }

    fn outcome_of(&self, run_id: u64) -> Option<(Outcome, Option<String>)> {
        self.journal
            .find(run_id)
            .map(|record| (record.result.outcome, record.result.reason))
    }
}
