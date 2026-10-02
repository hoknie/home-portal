use portal_widget::ResolvedSectionAppearance;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PublicSection {
    pub id: String,
    pub title: Option<String>,
    pub appearance: ResolvedSectionAppearance,
}
