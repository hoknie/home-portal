use std::collections::HashSet;

use portal_feature::FieldError;

use super::names::{NAME_RULE, valid_name};
use crate::types::{InputDeclaration, InputType, RawInput};

pub const TYPE_RULE: &str = "must be one of text, number, boolean, list and object";

pub fn decode_inputs(raw: &[RawInput], errors: &mut Vec<FieldError>) -> Vec<InputDeclaration> {
    let mut names = HashSet::new();
    let mut inputs = Vec::new();
    for (index, input) in raw.iter().enumerate() {
        let path = format!("inputs[{index}]");
        let name = input.name();
        if !valid_name(name) {
            let at = match input {
                RawInput::Name(_) => path.clone(),
                RawInput::Declared { .. } => format!("{path}.name"),
            };
            errors.push(FieldError::new(at, NAME_RULE));
            continue;
        }
        if !names.insert(name.to_string()) {
            errors.push(FieldError::new(path, "is declared twice"));
            continue;
        }
        let RawInput::Declared {
            input_type,
            default,
            description,
            ..
        } = input
        else {
            inputs.push(InputDeclaration::text(name));
            continue;
        };
        let Some(kind) = InputType::named(input_type.as_deref().unwrap_or("text")) else {
            errors.push(FieldError::new(format!("{path}.type"), TYPE_RULE));
            continue;
        };
        if let Some(value) = default.as_ref().filter(|value| !kind.fits(value)) {
            errors.push(FieldError::new(
                format!("{path}.default"),
                format!("must be {}, and {value} is not", kind.described()),
            ));
            continue;
        }
        inputs.push(InputDeclaration {
            name: name.to_string(),
            input_type: kind,
            default: default.clone(),
            description: description.clone().filter(|text| !text.trim().is_empty()),
        });
    }
    inputs
}
