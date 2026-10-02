use std::collections::HashSet;

use portal_feature::FieldError;

use super::names::{NAME_RULE, valid_name};
use crate::types::{InputDeclaration, InputType, OutputDeclaration, RawInput, RawOutput};

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

pub fn decode_outputs(raw: &[RawOutput], errors: &mut Vec<FieldError>) -> Vec<OutputDeclaration> {
    if raw.len() > OutputDeclaration::MOST {
        errors.push(FieldError::new(
            OutputDeclaration::FIELD,
            format!("holds at most {} outputs", OutputDeclaration::MOST),
        ));
    }
    let mut names = HashSet::new();
    let mut outputs = Vec::new();
    for (index, output) in raw.iter().enumerate() {
        let path = format!("{}[{index}]", OutputDeclaration::FIELD);
        if !valid_name(&output.name) {
            errors.push(FieldError::new(format!("{path}.name"), NAME_RULE));
        } else if !names.insert(output.name.as_str()) {
            errors.push(FieldError::new(
                format!("{path}.name"),
                "is used by another output of this workflow",
            ));
        }
        if output.value.trim().is_empty() {
            errors.push(FieldError::new(
                format!("{path}.value"),
                "must not be empty",
            ));
        }
        let description = output
            .description
            .clone()
            .filter(|text| !text.trim().is_empty());
        if description
            .as_ref()
            .is_some_and(|text| text.chars().count() > OutputDeclaration::LONGEST_DESCRIPTION)
        {
            errors.push(FieldError::new(
                format!("{path}.description"),
                format!(
                    "must be at most {} characters",
                    OutputDeclaration::LONGEST_DESCRIPTION
                ),
            ));
        }
        outputs.push(OutputDeclaration {
            name: output.name.clone(),
            value: output.value.clone(),
            description,
        });
    }
    outputs
}
