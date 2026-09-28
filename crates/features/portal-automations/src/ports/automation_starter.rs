use crate::types::Outcome;

pub trait AutomationStarter: Send + Sync {
    fn start(
        &self,
        automation: &str,
        fields: &[(String, String)],
        origin: &[String],
    ) -> Result<u64, String>;

    fn outcome_of(&self, run_id: u64) -> Option<(Outcome, Option<String>)>;
}
