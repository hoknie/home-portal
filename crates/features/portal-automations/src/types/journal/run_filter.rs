use crate::types::RunRecord;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RunFilter {
    pub automation: Option<String>,
    pub webhook: Option<String>,
    pub text: Option<String>,
}

impl RunFilter {
    pub const WEBHOOK_FIELD: &'static str = "webhook.id";

    pub fn keeps(&self, record: &RunRecord) -> bool {
        let by_automation = self
            .automation
            .as_deref()
            .is_none_or(|id| record.automation == id);
        let by_webhook = self.webhook.as_deref().is_none_or(|id| {
            record.automation == id
                || record
                    .fields
                    .iter()
                    .any(|(key, value)| key == Self::WEBHOOK_FIELD && value == id)
        });
        let by_text = self
            .text
            .as_deref()
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .is_none_or(|text| {
                let wanted = text.to_lowercase();
                record
                    .fields
                    .iter()
                    .map(|(_, value)| value)
                    .chain(record.arguments.iter())
                    .any(|value| value.to_lowercase().contains(&wanted))
            });
        by_automation && by_webhook && by_text
    }
}
