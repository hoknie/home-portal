use super::content_type_of;

pub fn looks_like_asset(path: &str) -> bool {
    !path.ends_with('/') && content_type_of(path).is_some()
}
