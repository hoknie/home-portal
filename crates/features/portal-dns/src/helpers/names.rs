pub const LONGEST_NAME: usize = 253;
pub const LONGEST_LABEL: usize = 63;

pub fn normalized(name: &str) -> Option<String> {
    let name = name.trim().trim_end_matches('.').to_ascii_lowercase();
    let valid = !name.is_empty()
        && name.len() <= LONGEST_NAME
        && name.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= LONGEST_LABEL
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label.chars().all(|character| {
                    character.is_ascii_alphanumeric() || character == '-' || character == '_'
                })
        });
    valid.then_some(name)
}

pub fn inside(name: &str, zone: &str) -> bool {
    name == zone || name.ends_with(&format!(".{zone}"))
}
