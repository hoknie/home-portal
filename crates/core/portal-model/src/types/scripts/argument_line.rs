use super::{ArgumentKind, ScriptArgument};

pub fn argument_of(line: &str) -> Result<ScriptArgument, String> {
    let line = line.trim();
    let (written, rest) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
    let (name, option, optional) = name_of(written)?;
    let rest = rest.trim_start();
    let (kind, default, description) = match rest.strip_prefix('<') {
        Some(inside) => {
            let (spec, after) = inside
                .split_once('>')
                .ok_or_else(|| "has a type that is not closed with >".to_string())?;
            let (kind_text, default) = match spec.split_once('=') {
                Some((kind_text, default)) => (kind_text.trim(), Some(default.trim().to_string())),
                None => (spec.trim(), None),
            };
            (kind_of(kind_text)?, default, after.trim())
        }
        None if option => (ArgumentKind::Flag, None, rest),
        None => (ArgumentKind::Text, None, rest),
    };
    if kind == ArgumentKind::Flag && !option {
        return Err(format!(
            "{name} is a flag, which must be an option such as --{name}"
        ));
    }
    if kind == ArgumentKind::Flag && default.is_some() {
        return Err(format!("{name} is a flag, and a flag takes no default"));
    }
    if let Some(default) = &default
        && !kind.accepts(default)
    {
        return Err(format!(
            "{name} has the default {default}, which is not a {}",
            kind_description(&kind)
        ));
    }
    Ok(ScriptArgument {
        required: !option && !optional && default.is_none(),
        name,
        option,
        kind,
        default,
        description: description.to_string(),
    })
}

fn name_of(written: &str) -> Result<(String, bool, bool), String> {
    let (base, option, optional) = match written.strip_prefix("--") {
        Some(base) => (base, true, false),
        None => match written.strip_suffix('?') {
            Some(base) => (base, false, true),
            None => (written, false, false),
        },
    };
    let valid = base
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_lowercase())
        && base.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || character == '_'
                || character == '-'
        });
    if !valid {
        return Err(format!(
            "names {written}, but a name is lowercase letters, digits, _ and -, starting with a letter, or -- before such a name"
        ));
    }
    let name = if option {
        written.to_string()
    } else {
        base.to_string()
    };
    Ok((name, option, optional))
}

fn kind_of(text: &str) -> Result<ArgumentKind, String> {
    match text {
        "text" => Ok(ArgumentKind::Text),
        "number" => Ok(ArgumentKind::Number),
        "flag" => Ok(ArgumentKind::Flag),
        choices if choices.contains('|') => {
            let choices: Vec<String> = choices
                .split('|')
                .map(|choice| choice.trim().to_string())
                .collect();
            if choices.iter().any(String::is_empty) {
                return Err("has a choice with an empty value".to_string());
            }
            Ok(ArgumentKind::Choice(choices))
        }
        other => Err(format!(
            "has the type {other}, but {other} is not a known type; use text, number, flag or choices such as a|b"
        )),
    }
}

fn kind_description(kind: &ArgumentKind) -> String {
    match kind {
        ArgumentKind::Choice(choices) => format!("choice of {}", choices.join(", ")),
        other => other.name().to_string(),
    }
}
