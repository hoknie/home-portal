use std::collections::BTreeMap;
use std::time::Duration;

use serde_json::Value;

use super::Block;

#[derive(Debug, Clone, PartialEq)]
pub enum WidgetSource {
    Workflow {
        id: String,
        inputs: BTreeMap<String, Value>,
    },
    Script {
        script: String,
        args: Vec<String>,
        timeout: Duration,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum WidgetTarget {
    Automation {
        id: String,
        fields: BTreeMap<String, String>,
    },
    Workflow {
        id: String,
        inputs: BTreeMap<String, Value>,
    },
    Refresh,
    Link(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WidgetAction {
    pub id: String,
    pub target: WidgetTarget,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomWidget {
    pub source: Option<WidgetSource>,
    pub refresh: Duration,
    pub blocks: Vec<Block>,
    pub actions: Vec<WidgetAction>,
}

impl CustomWidget {
    pub const KIND: &'static str = "custom";
    pub const DEFAULT_REFRESH: u64 = 300;
    pub const SHORTEST_REFRESH: i64 = 30;
    pub const LONGEST_REFRESH: i64 = 86_400;
    pub const MOST_BLOCKS: usize = 40;
    pub const LONGEST_SCRIPT: i64 = 300;
    pub const DEFAULT_SCRIPT_TIMEOUT: u64 = 30;
    pub fn action(&self, id: &str) -> Option<&WidgetAction> {
        self.actions.iter().find(|action| action.id == id)
    }
}
