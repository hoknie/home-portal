pub const LONGEST_NAME: usize = 63;
pub const NAME_RULE: &str =
    "must be 1 to 63 lowercase letters, digits and _, starting with a letter";

pub fn valid_name(name: &str) -> bool {
    let mut characters = name.chars();
    name.len() <= LONGEST_NAME
        && characters
            .next()
            .is_some_and(|first| first.is_ascii_lowercase())
        && characters.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

pub fn within(value: i64, minimum: i64, maximum: i64) -> bool {
    (minimum..=maximum).contains(&value)
}

pub fn between_rule(minimum: i64, maximum: i64) -> String {
    format!("must be {minimum} to {maximum}")
}
