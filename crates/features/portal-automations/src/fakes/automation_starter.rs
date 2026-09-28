use std::sync::Mutex;

use crate::ports::AutomationStarter;
use crate::types::Outcome;

pub type Started = (String, Vec<(String, String)>, Vec<String>);

#[derive(Default)]
pub struct FakeStarter {
    pub started: Mutex<Vec<Started>>,
}

impl AutomationStarter for FakeStarter {
    fn start(
        &self,
        automation: &str,
        fields: &[(String, String)],
        origin: &[String],
    ) -> Result<u64, String> {
        if origin.iter().any(|id| id == automation) {
            return Err(format!(
                "the automation {automation} already started this run"
            ));
        }
        let mut started = self.started.lock().unwrap();
        started.push((automation.to_string(), fields.to_vec(), origin.to_vec()));
        Ok(started.len() as u64)
    }

    fn outcome_of(&self, _run_id: u64) -> Option<(Outcome, Option<String>)> {
        Some((Outcome::Succeeded, None))
    }
}
