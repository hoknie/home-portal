use std::collections::BTreeMap;

use serde_json::Value;

use crate::types::Workflow;

pub fn bind_inputs(
    workflow: &Workflow,
    given: Vec<(String, Value)>,
) -> Result<BTreeMap<String, Value>, (String, String)> {
    let mut given: BTreeMap<String, Value> = given.into_iter().collect();
    let mut bound = BTreeMap::new();
    for input in &workflow.inputs {
        let value = match given.remove(&input.name) {
            Some(Value::String(text)) if text.is_empty() && input.default.is_some() => {
                input.default.clone()
            }
            Some(value) => Some(value),
            None => input.default.clone(),
        };
        let Some(value) = value else { continue };
        let coerced = input
            .input_type
            .coerce(value)
            .map_err(|message| (input.name.clone(), message))?;
        bound.insert(input.name.clone(), coerced);
    }
    Ok(bound)
}

pub fn input_problem((name, message): (String, String)) -> String {
    format!("the input {name} {message}")
}
