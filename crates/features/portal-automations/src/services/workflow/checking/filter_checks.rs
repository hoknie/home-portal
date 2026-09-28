use serde_json::Value;

use crate::types::{ArgumentType, FilterCall, FilterDescription, ValueType, filter_named};

pub fn call_problem(call: &FilterCall) -> Option<String> {
    let Some(description) = filter_named(&call.name) else {
        return Some(format!(
            "names the filter {}, which does not exist",
            call.name
        ));
    };
    arguments_problem(description, &call.arguments)
}

pub fn arguments_problem(description: &FilterDescription, arguments: &[Value]) -> Option<String> {
    let fewest = description.fewest_arguments();
    let most = description.arguments.len();
    if arguments.len() < fewest || arguments.len() > most {
        let names: Vec<&str> = description
            .arguments
            .iter()
            .map(|argument| argument.name)
            .collect();
        let wanted = match (fewest, most) {
            (1, 1) => "1 argument".to_string(),
            (fewest, most) if fewest == most => format!("{most} arguments"),
            (fewest, most) => format!("{fewest} to {most} arguments"),
        };
        return Some(format!(
            "{} takes {wanted} ({}) and got {}",
            description.name,
            names.join(", "),
            arguments.len()
        ));
    }
    description
        .arguments
        .iter()
        .zip(arguments)
        .find_map(|(argument, value)| {
            let fits = match argument.argument_type {
                ArgumentType::Text => value.is_string(),
                ArgumentType::Number => value.is_number(),
                _ => true,
            };
            (!fits).then(|| {
                format!(
                    "the argument {} of {} must be {}",
                    argument.name,
                    description.name,
                    if argument.argument_type == ArgumentType::Text {
                        "quoted text"
                    } else {
                        "a number"
                    }
                )
            })
        })
}

pub fn chain_problem(start: ValueType, filters: &[FilterCall]) -> Option<String> {
    let mut current = start;
    for call in filters {
        if let Some(problem) = call_problem(call) {
            return Some(problem);
        }
        let description = filter_named(&call.name)?;
        let certain = !matches!(current, ValueType::Any | ValueType::Null);
        let over_list = current == ValueType::List && description.element;
        if certain && !over_list && !description.takes(current) {
            return Some(format!(
                "{} takes {} and got {}",
                call.name,
                description.accepted(),
                current.described()
            ));
        }
        current = match (call.name.as_str(), description.gives) {
            ("slice", _) => current,
            _ if over_list && !description.takes(ValueType::List) => ValueType::List,
            (_, gives) => gives,
        };
    }
    None
}
