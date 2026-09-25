use portal_feature::ApiError;

pub const NOT_FOUND_BODY: &str = "not found";

pub async fn not_found() -> ApiError {
    ApiError::NotFound(NOT_FOUND_BODY)
}
