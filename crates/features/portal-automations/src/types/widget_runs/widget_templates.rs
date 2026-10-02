use std::collections::BTreeMap;
use std::sync::Arc;

use serde_json::{Value, json};

use crate::services::{Frame, Secrets, render_text, render_value};
use crate::types::EventValues;

pub struct WidgetTemplates {
    frame: Frame,
}

impl WidgetTemplates {
    pub fn of(
        data: &Value,
        id: &str,
        title: Option<&str>,
        fetched_at: Option<String>,
    ) -> WidgetTemplates {
        let mut frame = Frame::new(
            Arc::new(EventValues::default()),
            BTreeMap::new(),
            Arc::new(Secrets::new(Arc::new(|_| None))),
        );
        frame.widget = Some(json!({
            "data": data,
            "widget": { "id": id, "title": title },
            "fetched_at": fetched_at,
        }));
        WidgetTemplates { frame }
    }

    pub fn text(&self, template: &str) -> Result<String, String> {
        render_text(template, &self.frame)
    }

    pub fn value(&self, template: &str) -> Result<Value, String> {
        render_value(template, &self.frame)
    }

    pub fn set_item(&mut self, item: Option<(Value, usize)>) {
        self.frame.transform_item = item;
    }
}
