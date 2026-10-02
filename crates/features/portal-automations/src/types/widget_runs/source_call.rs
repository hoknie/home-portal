use std::collections::BTreeMap;
use std::time::Duration;

use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum SourceCall {
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

impl SourceCall {
    pub const WIDGET_PREFIX: &'static str = "widget:";
    pub const LONGEST_SOURCE: Duration = Duration::from_secs(300);

    pub fn run_key(widget: &str) -> String {
        format!("{}{widget}", Self::WIDGET_PREFIX)
    }
}
