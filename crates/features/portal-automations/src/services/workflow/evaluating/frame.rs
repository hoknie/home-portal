use std::collections::BTreeMap;
use std::sync::Arc;

use serde_json::Value;

use super::rendered::Collector;
use super::secrets::Secrets;

#[derive(Clone)]
pub struct Frame {
    pub event: Arc<Vec<(String, String)>>,
    pub inputs: BTreeMap<String, Value>,
    pub vars: BTreeMap<String, Value>,
    pub steps: BTreeMap<String, Value>,
    pub loop_item: Option<(Value, usize)>,
    pub transform_item: Option<(Value, usize)>,
    pub portal: Option<Value>,
    pub secrets: Arc<Secrets>,
    pub rendered: Option<Collector>,
}

impl Frame {
    pub fn new(
        event: Arc<Vec<(String, String)>>,
        inputs: BTreeMap<String, Value>,
        secrets: Arc<Secrets>,
    ) -> Frame {
        Frame {
            event,
            inputs,
            vars: BTreeMap::new(),
            steps: BTreeMap::new(),
            loop_item: None,
            transform_item: None,
            portal: None,
            secrets,
            rendered: None,
        }
    }

    pub fn lookup(&self, name: &str) -> Result<Value, String> {
        let mut parts = name.split('.');
        let namespace = parts.next().unwrap_or_default();
        let rest: Vec<&str> = parts.collect();
        if namespace == "portal" {
            return Ok(portal_lookup(self.portal.as_ref(), &rest));
        }
        match (namespace, &self.transform_item) {
            ("item", Some((item, _))) => return Ok(walk(item.clone(), &rest)),
            ("index", Some((_, index))) => return Ok(Value::from(*index)),
            ("item" | "index", None) => return Ok(Value::Null),
            _ => {}
        }
        let Some((first, path)) = rest.split_first() else {
            return Ok(Value::Null);
        };
        let found = match namespace {
            "event" => {
                let field = rest.join(".");
                return Ok(self
                    .event
                    .iter()
                    .find(|(name, _)| *name == field)
                    .map(|(_, value)| Value::String(value.clone()))
                    .unwrap_or(Value::Null));
            }
            "inputs" => self.inputs.get(*first).cloned(),
            "vars" => self.vars.get(*first).cloned(),
            "steps" => self.steps.get(*first).cloned(),
            "loop" => match (*first, &self.loop_item) {
                ("item", Some((item, _))) => Some(item.clone()),
                ("index", Some((_, index))) => Some(Value::from(*index)),
                _ => None,
            },
            "secrets" => return self.secrets.reveal(first).map(Value::String),
            _ => None,
        };
        Ok(found.map_or(Value::Null, |value| walk(value, path)))
    }
}

pub fn walk(value: Value, path: &[&str]) -> Value {
    path.iter().fold(value, |current, part| match current {
        Value::Object(mut map) => map
            .remove(*part)
            .or_else(|| {
                let lowered = part.to_ascii_lowercase();
                map.into_iter()
                    .find(|(key, _)| key.to_ascii_lowercase() == lowered)
                    .map(|(_, value)| value)
            })
            .unwrap_or(Value::Null),
        Value::Array(mut items) => part
            .parse::<usize>()
            .ok()
            .filter(|index| *index < items.len())
            .map(|index| items.swap_remove(index))
            .unwrap_or(Value::Null),
        _ => Value::Null,
    })
}

fn portal_lookup(portal: Option<&Value>, path: &[&str]) -> Value {
    let Some(portal) = portal else {
        return Value::Null;
    };
    match path {
        ["services", id, rest @ ..] => portal["services"]
            .as_array()
            .and_then(|services| services.iter().find(|service| service["id"] == *id))
            .map_or(Value::Null, |service| walk(service.clone(), rest)),
        _ => walk(portal.clone(), path),
    }
}
