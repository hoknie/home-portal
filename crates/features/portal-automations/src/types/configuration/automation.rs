use portal_feature::FieldError;

use super::{RawAutomation, RawRun, RunSettings, Trigger, WorkflowCall};
use crate::helpers::{check_tags, unknown_placeholders};
use crate::types::{Catalogue, InputValue};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Automation {
    pub id: String,
    pub title: String,
    pub enabled: bool,
    pub tags: Vec<String>,
    pub cooldown_seconds: u64,
    pub trigger: Trigger,
    pub run: RunSettings,
    pub workflow: Option<WorkflowCall>,
}

impl Automation {
    pub const UNKNOWN: &'static str = "no such automation";
    pub const TAKEN_ID: &'static str = "is used by another automation";
    pub const LONGEST_ID: usize = 63;
    pub const RESERVED_IDS: [&'static str; 4] = ["runs", "catalogue", "scripts", "schedule"];
    pub const BOTH_ACTIONS: &'static str =
        "an automation runs either a script (run) or a workflow, not both";
    pub const NO_ACTION: &'static str = "is required: a script to run, or a workflow";

    pub fn decode(raw: &RawAutomation) -> Result<Automation, Vec<FieldError>> {
        let mut errors = Vec::new();
        if Self::RESERVED_IDS.contains(&raw.id.as_str()) {
            errors.push(FieldError::new(
                "id",
                format!(
                    "is reserved by the interface; {} cannot be used",
                    Self::RESERVED_IDS.join(", ")
                ),
            ));
        } else if !Self::valid_id(&raw.id) {
            errors.push(FieldError::new(
                "id",
                format!(
                    "must be 1 to {} lowercase letters, digits and -",
                    Self::LONGEST_ID
                ),
            ));
        }
        if raw.title.trim().is_empty() {
            errors.push(FieldError::new("title", "must not be empty"));
        }
        errors.extend(check_tags(&raw.tags));
        let cooldown = raw.cooldown_seconds.unwrap_or(0);
        if cooldown < 0 {
            errors.push(FieldError::new("cooldown_seconds", "must not be negative"));
        }
        let trigger = Trigger::decode(&raw.when).map_err(|found| errors.extend(found));
        let (run, workflow) = Self::decode_action(raw, &mut errors);
        if let Ok(trigger) = &trigger {
            let allowed = Catalogue::fields_of(trigger.event);
            let webhook = trigger.event == portal_feature::EventName::WebhookReceived;
            for (field, template) in Self::templates_of(&run, workflow.as_ref()) {
                let unknown: Vec<String> = unknown_placeholders(template, &allowed)
                    .into_iter()
                    .filter(|name| {
                        !(webhook && name.starts_with(portal_feature::PortalEvent::VARIABLE_PREFIX))
                    })
                    .collect();
                if let Some(name) = unknown.first() {
                    errors.push(FieldError::new(
                        field,
                        format!(
                            "names {{{{{name}}}}}, which is not a field of {}; its fields are {}",
                            trigger.event.name(),
                            allowed.join(", ")
                        ),
                    ));
                }
            }
        }
        match (trigger, run) {
            (Ok(trigger), Some(run)) if errors.is_empty() => Ok(Automation {
                id: raw.id.clone(),
                title: raw.title.clone(),
                enabled: raw.enabled.unwrap_or(true),
                tags: raw.tags.clone(),
                cooldown_seconds: cooldown as u64,
                trigger,
                run,
                workflow,
            }),
            _ => Err(errors),
        }
    }

    pub fn decode_action(
        raw: &RawAutomation,
        errors: &mut Vec<FieldError>,
    ) -> (Option<RunSettings>, Option<WorkflowCall>) {
        Self::decode_either(
            raw.run.as_ref(),
            (raw.workflow.as_ref(), raw.inputs.as_ref()),
            errors,
        )
    }

    pub fn decode_either(
        run: Option<&RawRun>,
        (workflow, inputs): (
            Option<&String>,
            Option<&std::collections::BTreeMap<String, InputValue>>,
        ),
        errors: &mut Vec<FieldError>,
    ) -> (Option<RunSettings>, Option<WorkflowCall>) {
        match (run, workflow) {
            (Some(_), Some(_)) => {
                errors.push(FieldError::new("workflow", Self::BOTH_ACTIONS));
                (None, None)
            }
            (None, None) => {
                errors.push(FieldError::new("run", Self::NO_ACTION));
                (None, None)
            }
            (Some(run), None) => (
                RunSettings::decode(run)
                    .map_err(|found| errors.extend(found))
                    .ok(),
                None,
            ),
            (None, Some(id)) => {
                if id.trim().is_empty() {
                    errors.push(FieldError::new("workflow", "must name a workflow"));
                }
                let call = WorkflowCall {
                    id: id.clone(),
                    inputs: inputs.cloned().unwrap_or_default().into_iter().collect(),
                };
                (Some(RunSettings::default()), Some(call))
            }
        }
    }

    pub fn templates_of<'a>(
        run: &'a Option<RunSettings>,
        workflow: Option<&'a WorkflowCall>,
    ) -> Vec<(String, &'a str)> {
        match workflow {
            Some(call) => call.templates().collect(),
            None => run
                .as_ref()
                .map(|run| {
                    run.args
                        .iter()
                        .enumerate()
                        .map(|(index, argument)| (format!("run.args[{index}]"), argument.as_str()))
                        .collect()
                })
                .unwrap_or_default(),
        }
    }

    pub fn valid_id(id: &str) -> bool {
        !id.is_empty()
            && id.len() <= Self::LONGEST_ID
            && id
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    }
}
