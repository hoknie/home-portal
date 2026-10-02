use std::sync::Arc;

use portal_feature::WidgetNeed;
use serde_json::Value;

pub type NeedsOf = Arc<dyn Fn(&str, &Value) -> Vec<WidgetNeed> + Send + Sync>;

pub fn no_needs() -> NeedsOf {
    Arc::new(|_: &str, _: &Value| Vec::new())
}
