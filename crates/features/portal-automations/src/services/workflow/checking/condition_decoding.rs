use portal_feature::FieldError;

use crate::types::{Condition, Operator, RawCondition};

pub const MOST_CONDITIONS: usize = 20;
pub const DEEPEST_GROUPS: usize = 3;

pub fn decode_condition(
    raw: &RawCondition,
    path: &str,
    groups: usize,
    errors: &mut Vec<FieldError>,
) -> Option<Condition> {
    match (&raw.all, &raw.any) {
        (Some(_), Some(_)) => {
            errors.push(FieldError::new(path, "must be either all or any, not both"));
            None
        }
        (Some(inner), None) => {
            decode_group(inner, &format!("{path}.all"), groups, errors).map(Condition::All)
        }
        (None, Some(inner)) => {
            decode_group(inner, &format!("{path}.any"), groups, errors).map(Condition::Any)
        }
        (None, None) => decode_comparison(raw, path, errors),
    }
}

fn decode_group(
    inner: &[RawCondition],
    path: &str,
    groups: usize,
    errors: &mut Vec<FieldError>,
) -> Option<Vec<Condition>> {
    if groups >= DEEPEST_GROUPS {
        errors.push(FieldError::new(
            path,
            format!("groups nest at most {DEEPEST_GROUPS} deep"),
        ));
        return None;
    }
    if inner.is_empty() || inner.len() > MOST_CONDITIONS {
        errors.push(FieldError::new(
            path,
            format!("must hold 1 to {MOST_CONDITIONS} conditions"),
        ));
        return None;
    }
    let decoded: Vec<Option<Condition>> = inner
        .iter()
        .enumerate()
        .map(|(index, condition)| {
            decode_condition(condition, &format!("{path}[{index}]"), groups + 1, errors)
        })
        .collect();
    decoded.into_iter().collect()
}

fn decode_comparison(
    raw: &RawCondition,
    path: &str,
    errors: &mut Vec<FieldError>,
) -> Option<Condition> {
    let Some(left) = raw.left.clone() else {
        errors.push(FieldError::new(format!("{path}.left"), "is required"));
        return None;
    };
    let Some(operator) = raw.op.as_deref().and_then(Operator::of) else {
        let names: Vec<&str> = Operator::ALL
            .iter()
            .map(|operator| operator.name())
            .collect();
        errors.push(FieldError::new(
            format!("{path}.op"),
            format!("must be one of {}", names.join(", ")),
        ));
        return None;
    };
    if operator.takes_right() && raw.right.is_none() {
        errors.push(FieldError::new(format!("{path}.right"), "is required"));
        return None;
    }
    Some(Condition::Compare {
        left,
        operator,
        right: raw.right.clone().filter(|_| operator.takes_right()),
    })
}
