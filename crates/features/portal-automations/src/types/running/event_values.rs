use serde_json::{Map, Value};

use crate::helpers::{BODY, body_path, render, text_of, walk};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EventValues {
    pub fields: Vec<(String, String)>,
    pub body: Option<Value>,
}

impl EventValues {
    pub fn new(fields: Vec<(String, String)>, body: Option<Value>) -> EventValues {
        EventValues { fields, body }
    }

    pub fn field(&self, name: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    pub fn value(&self, name: &str) -> Option<Value> {
        match body_path(name) {
            Some(path) => self.body.clone().map(|body| walk(body, &path)),
            None => self
                .field(name)
                .map(|value| Value::String(value.to_string())),
        }
    }

    pub fn text(&self, name: &str) -> Option<String> {
        self.value(name).map(|value| text_of(&value))
    }

    pub fn render(&self, template: &str) -> String {
        render(template, |name| self.text(name))
    }

    pub fn resolve(&self, template: &str) -> Value {
        let trimmed = template.trim();
        let whole = trimmed
            .strip_prefix("{{")
            .and_then(|rest| rest.strip_suffix("}}"))
            .filter(|name| !name.contains("{{") && !name.contains("}}"));
        whole
            .and_then(|name| self.value(name))
            .unwrap_or_else(|| Value::String(self.render(template)))
    }

    pub fn input(&self) -> String {
        let mut input: Map<String, Value> = self
            .fields
            .iter()
            .map(|(key, value)| (key.clone(), Value::String(value.clone())))
            .collect();
        if let Some(body) = &self.body {
            input.insert(BODY.to_string(), body.clone());
        }
        Value::Object(input).to_string()
    }
}

impl From<Vec<(String, String)>> for EventValues {
    fn from(fields: Vec<(String, String)>) -> EventValues {
        EventValues::new(fields, None)
    }
}
