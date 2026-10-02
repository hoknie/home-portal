use serde::Deserialize;
use serde_json::Value;

use crate::usecases::PreviewAsked;

#[derive(Debug, Clone, Deserialize)]
pub struct WidgetPreviewRequest {
    #[serde(default)]
    pub id: Option<String>,
    pub settings: Value,
    #[serde(default)]
    pub run: bool,
    #[serde(default)]
    pub sample: Option<Value>,
}

impl WidgetPreviewRequest {
    pub fn asked(self) -> PreviewAsked {
        PreviewAsked {
            id: self.id,
            settings: self.settings,
            run: self.run,
            sample: self.sample,
        }
    }
}
