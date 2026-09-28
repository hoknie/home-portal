use portal_feature::FieldError;

use super::names::{between_rule, within};
use crate::helpers::script_shape_problem;
use crate::types::{HttpStep, Invocation, METHODS, RawStep, RunSettings, StepKind};

pub const URL_RULE: &str = "must be an http or https address";
pub const LEGACY_TELEGRAM: &str = "telegram";
pub const HEADER_RULE: &str = "is set by the portal and cannot be given";
pub const SAMPLE_RULE: &str = "must be JSON, an example of the answer";
pub const SAMPLE_SIZE_RULE: &str = "must be at most 16 KiB";

pub fn decode_http(raw: &RawStep, path: &str, errors: &mut Vec<FieldError>) -> Option<StepKind> {
    let before = errors.len();
    let method = raw
        .method
        .clone()
        .unwrap_or_else(|| "GET".to_string())
        .to_ascii_uppercase();
    if !METHODS.contains(&method.as_str()) {
        errors.push(FieldError::new(
            format!("{path}.method"),
            format!("must be one of {}", METHODS.join(", ")),
        ));
    }
    if let Some(sample) = &raw.response_sample {
        if sample.len() > HttpStep::LARGEST_SAMPLE {
            errors.push(FieldError::new(
                format!("{path}.response_sample"),
                SAMPLE_SIZE_RULE,
            ));
        } else if serde_json::from_str::<serde_json::Value>(sample).is_err() {
            errors.push(FieldError::new(
                format!("{path}.response_sample"),
                SAMPLE_RULE,
            ));
        }
    }
    let url = raw.url.clone().unwrap_or_default();
    if url.trim().is_empty() {
        errors.push(FieldError::new(format!("{path}.url"), "is required"));
    } else if !url.trim_start().starts_with("{{") && !literal_http(&url) {
        errors.push(FieldError::new(format!("{path}.url"), URL_RULE));
    }
    let headers: Vec<(String, String)> = raw
        .headers
        .clone()
        .unwrap_or_default()
        .into_iter()
        .collect();
    for (name, _) in &headers {
        if HttpStep::FORBIDDEN_HEADERS.contains(&name.to_ascii_lowercase().as_str()) {
            errors.push(FieldError::new(
                format!("{path}.headers.{name}"),
                HEADER_RULE,
            ));
        } else if name.is_empty()
            || !name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            errors.push(FieldError::new(
                format!("{path}.headers.{name}"),
                "must be a header name of letters, digits, - and _",
            ));
        }
    }
    let body = match &raw.body {
        None => None,
        Some(toml::Value::String(text)) => Some(text.clone()),
        Some(_) => {
            errors.push(FieldError::new(format!("{path}.body"), "must be text"));
            None
        }
    };
    let timeout = timeout_of(
        raw.timeout_seconds,
        (HttpStep::DEFAULT_TIMEOUT, HttpStep::LONGEST_TIMEOUT),
        path,
        errors,
    );
    (errors.len() == before).then(|| {
        StepKind::Http(HttpStep {
            method,
            url,
            headers,
            body,
            timeout_seconds: timeout,
            fail_on_error: raw.fail_on_error.unwrap_or(true),
        })
    })
}

pub fn literal_http(url: &str) -> bool {
    let lowered = url.trim_start().to_ascii_lowercase();
    lowered.starts_with("http://") || lowered.starts_with("https://")
}

pub fn decode_script(raw: &RawStep, path: &str, errors: &mut Vec<FieldError>) -> Option<StepKind> {
    let before = errors.len();
    let script = raw.script.clone().unwrap_or_default();
    if let Some(problem) = script_shape_problem(&script) {
        errors.push(FieldError::new(format!("{path}.script"), problem));
    }
    let timeout = timeout_of(
        raw.timeout_seconds,
        (RunSettings::DEFAULT_TIMEOUT, RunSettings::LONGEST_TIMEOUT),
        path,
        errors,
    );
    let env: Vec<(String, String)> = raw.env.clone().unwrap_or_default().into_iter().collect();
    for (name, _) in &env {
        if let Some(problem) = variable_problem(name) {
            errors.push(FieldError::new(format!("{path}.env.{name}"), problem));
        }
    }
    (errors.len() == before).then(|| StepKind::Script {
        run: RunSettings {
            script,
            args: raw.args.clone().unwrap_or_default(),
            timeout_seconds: timeout,
        },
        env,
        stdin: raw.stdin.clone().filter(|stdin| !stdin.trim().is_empty()),
    })
}

pub const VARIABLE_RULE: &str =
    "must be 1 to 64 capital letters, digits and _, starting with a letter";
pub const RESERVED_VARIABLE: &str = "is set by the portal and cannot be given";

pub fn variable_problem(name: &str) -> Option<&'static str> {
    let mut characters = name.chars();
    let valid = name.len() <= 64
        && characters
            .next()
            .is_some_and(|first| first.is_ascii_uppercase())
        && characters.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
    if !valid {
        return Some(VARIABLE_RULE);
    }
    let reserved =
        name.starts_with(Invocation::VARIABLE_PREFIX) || Invocation::PASSED_THROUGH.contains(&name);
    reserved.then_some(RESERVED_VARIABLE)
}

pub fn decode_notify(raw: &RawStep, path: &str, errors: &mut Vec<FieldError>) -> Option<StepKind> {
    let channel = raw
        .channel
        .clone()
        .filter(|channel| !channel.trim().is_empty())
        .or_else(|| (raw.kind == LEGACY_TELEGRAM).then(|| LEGACY_TELEGRAM.to_string()));
    let title = raw.title.clone().filter(|title| !title.trim().is_empty());
    match raw.text.clone().filter(|text| !text.trim().is_empty()) {
        Some(text) => Some(StepKind::Notify {
            channel,
            title,
            text,
        }),
        None => {
            errors.push(FieldError::new(format!("{path}.text"), "is required"));
            None
        }
    }
}

pub fn decode_automation(
    raw: &RawStep,
    path: &str,
    errors: &mut Vec<FieldError>,
) -> Option<StepKind> {
    match raw.automation.clone().filter(|id| !id.trim().is_empty()) {
        Some(automation) => Some(StepKind::Automation {
            automation,
            fields: raw.fields.clone().unwrap_or_default().into_iter().collect(),
            wait: raw.wait.unwrap_or(false),
        }),
        None => {
            errors.push(FieldError::new(format!("{path}.automation"), "is required"));
            None
        }
    }
}

pub fn decode_service(raw: &RawStep, path: &str, errors: &mut Vec<FieldError>) -> Option<String> {
    let service = raw
        .service
        .clone()
        .filter(|service| !service.trim().is_empty());
    if service.is_none() {
        errors.push(FieldError::new(format!("{path}.service"), "is required"));
    }
    service
}

fn timeout_of(
    given: Option<i64>,
    (default, longest): (u64, u64),
    path: &str,
    errors: &mut Vec<FieldError>,
) -> u64 {
    let timeout = given.unwrap_or(default as i64);
    if !within(timeout, 1, longest as i64) {
        errors.push(FieldError::new(
            format!("{path}.timeout_seconds"),
            between_rule(1, longest as i64),
        ));
        return default;
    }
    timeout as u64
}
