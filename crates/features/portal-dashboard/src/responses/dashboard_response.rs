use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{SectionView, WidgetView};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DashboardResponse {
    pub sections: Vec<SectionView>,
    pub widgets: Vec<WidgetView>,
}
