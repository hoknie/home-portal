use crate::services::workflow::{chain_problem, placeholders_in};
use crate::types::ValueType;

pub fn template_problem(
    template: &str,
    allows: &dyn Fn(&str) -> Result<ValueType, String>,
) -> Option<String> {
    placeholders_in(template)
        .into_iter()
        .find_map(|placeholder| {
            let argument = placeholder.filters.iter().flatten().find_map(|call| {
                call.names.iter().find_map(|(_, name)| {
                    allows(name).err().map(|problem| {
                        format!(
                            "{{{{{}}}}}: the argument of {} {problem}",
                            placeholder.name, call.name
                        )
                    })
                })
            });
            argument.or_else(|| match allows(placeholder.name) {
                Err(problem) => Some(problem),
                Ok(kind) => match &placeholder.filters {
                    Ok(filters) => chain_problem(kind, filters),
                    Err(message) => Some(message.clone()),
                }
                .map(|problem| format!("{{{{{}}}}}: {problem}", placeholder.name)),
            })
        })
}
