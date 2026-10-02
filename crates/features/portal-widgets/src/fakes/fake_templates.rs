use serde_json::{Value, json};

use crate::ports::{NameKind, TemplateContext, WidgetTemplates};

pub struct FakeTemplates;

fn names(template: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        let Some(end) = rest[start..].find("}}") else {
            break;
        };
        let inside = &rest[start + 2..start + end];
        found.push(
            inside
                .split('|')
                .next()
                .unwrap_or_default()
                .trim()
                .to_string(),
        );
        rest = &rest[start + end + 2..];
    }
    found
}

impl WidgetTemplates for FakeTemplates {
    fn problem(
        &self,
        template: &str,
        allows: &dyn Fn(&str) -> Result<NameKind, String>,
    ) -> Option<String> {
        names(template).iter().find_map(|name| allows(name).err())
    }

    fn open(
        &self,
        data: &Value,
        id: &str,
        title: Option<&str>,
        fetched_at: Option<String>,
    ) -> Box<dyn TemplateContext> {
        Box::new(FakeContext {
            root: json!({ "data": data, "widget": { "id": id, "title": title }, "fetched_at": fetched_at }),
            item: None,
        })
    }
}

pub struct FakeContext {
    root: Value,
    item: Option<(Value, usize)>,
}

impl FakeContext {
    fn lookup(&self, name: &str) -> Value {
        let mut parts = name.split('.');
        let namespace = parts.next().unwrap_or_default();
        let mut current = match (namespace, &self.item) {
            ("item", Some((item, _))) => item.clone(),
            ("index", Some((_, index))) => return json!(index),
            _ => self.root[namespace].clone(),
        };
        for key in parts {
            current = match key.parse::<usize>() {
                Ok(index) if current.is_array() => current[index].clone(),
                _ => current[key].clone(),
            };
        }
        current
    }
}

fn shown(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

impl TemplateContext for FakeContext {
    fn text(&self, template: &str) -> Result<String, String> {
        let mut out = String::new();
        let mut rest = template;
        while let Some(start) = rest.find("{{") {
            let Some(end) = rest[start..].find("}}") else {
                break;
            };
            out.push_str(&rest[..start]);
            let name = rest[start + 2..start + end]
                .split('|')
                .next()
                .unwrap_or_default()
                .trim();
            out.push_str(&shown(&self.lookup(name)));
            rest = &rest[start + end + 2..];
        }
        out.push_str(rest);
        Ok(out)
    }

    fn value(&self, template: &str) -> Result<Value, String> {
        let trimmed = template.trim();
        let single = names(trimmed);
        if single.len() == 1 && trimmed.starts_with("{{") && trimmed.ends_with("}}") {
            return Ok(self.lookup(&single[0]));
        }
        self.text(template).map(Value::String)
    }

    fn set_item(&mut self, item: Option<(Value, usize)>) {
        self.item = item;
    }
}
