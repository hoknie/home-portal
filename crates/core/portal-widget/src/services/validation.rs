use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

use portal_feature::FieldError;
use portal_model::ServiceId;

use crate::types::WidgetInstance;
use portal_feature::WidgetProvider;

pub const SECTION: &str = "dashboard.widgets";

pub fn check_instances(
    instances: &[WidgetInstance],
    providers: &BTreeMap<&'static str, Arc<dyn WidgetProvider>>,
) -> Vec<FieldError> {
    let mut errors = Vec::new();
    let mut seen = HashSet::new();
    for (index, instance) in instances.iter().enumerate() {
        let field = format!("{SECTION}[{index}]");
        let Some(provider) = providers.get(instance.kind.as_str()) else {
            continue;
        };
        match &instance.id {
            None => errors.push(FieldError::new(
                format!("{field}.id"),
                format!(
                    "a {} widget needs an id, so that its data can be asked for",
                    instance.kind
                ),
            )),
            Some(id) => {
                if let Err(problem) = ServiceId::parse(id) {
                    errors.push(FieldError::new(format!("{field}.id"), problem.to_string()));
                }
                if !seen.insert(id.as_str()) {
                    errors.push(FieldError::new(
                        format!("{field}.id"),
                        "is used by another widget",
                    ));
                }
            }
        }
        let prefix = match &instance.id {
            Some(id) => format!("{SECTION}.{id}.settings."),
            None => format!("{field}.settings."),
        };
        errors.extend(
            provider
                .check(&instance.settings)
                .into_iter()
                .map(|error| error.prefixed(&prefix)),
        );
    }
    errors
}
