use serde::Deserialize;

use super::{SectionEntry, WidgetInstance};

#[derive(Debug, Default, Deserialize)]
pub struct DashboardTable {
    #[serde(default)]
    pub sections: Vec<SectionEntry>,
    #[serde(default)]
    pub widgets: Vec<WidgetInstance>,
}
