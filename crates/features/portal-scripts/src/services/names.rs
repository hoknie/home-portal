use portal_feature::ApiError;
use portal_model::ScriptPath;

pub const NAME_RULE: &str = "must be letters, digits, ., _ and -, and must not start with .";

pub fn valid_part(part: &str) -> bool {
    !part.is_empty()
        && !part.starts_with('.')
        && part.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')
        })
}

pub fn script_path(field: &str, text: &str) -> Result<ScriptPath, ApiError> {
    let path =
        ScriptPath::parse(text).map_err(|problem| ApiError::invalid(field, problem.message()))?;
    if !path.text().split('/').all(valid_part) {
        return Err(ApiError::invalid(field, NAME_RULE));
    }
    Ok(path)
}

pub fn folder_name(field: &str, text: &str) -> Result<String, ApiError> {
    if text.contains('/') || !valid_part(text) {
        return Err(ApiError::invalid(field, NAME_RULE));
    }
    Ok(text.to_string())
}
