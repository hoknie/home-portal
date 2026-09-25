use portal_feature::FieldError;

pub const LONGEST_TAG: usize = 40;
pub const MOST_TAGS: usize = 20;

pub fn check_tags(tags: &[String]) -> Vec<FieldError> {
    let mut errors = Vec::new();
    if tags.len() > MOST_TAGS {
        errors.push(FieldError::new(
            "tags",
            format!("must hold at most {MOST_TAGS} tags"),
        ));
    }
    for (index, tag) in tags.iter().enumerate() {
        let trimmed = tag.trim();
        if trimmed.is_empty() || trimmed != tag || trimmed.chars().count() > LONGEST_TAG {
            errors.push(FieldError::new(
                format!("tags[{index}]"),
                format!("must be 1 to {LONGEST_TAG} characters without spaces at either end"),
            ));
        } else if tags[..index]
            .iter()
            .any(|earlier| earlier.to_lowercase() == tag.to_lowercase())
        {
            errors.push(FieldError::new(format!("tags[{index}]"), "is named twice"));
        }
    }
    errors
}
