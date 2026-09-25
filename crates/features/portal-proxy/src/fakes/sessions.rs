use axum::http::HeaderMap;
use axum::http::header::COOKIE;
use portal_feature::{ApiError, Gate, Principal};

pub struct Sessions;

impl Sessions {
    pub const COOKIE: &'static str = "home_portal_session=anna-token";
    pub const USER: &'static str = "anna";
}

impl Gate for Sessions {
    fn admit(&self, headers: &HeaderMap) -> Result<Principal, ApiError> {
        headers
            .get_all(COOKIE)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .any(|value| value.contains(Self::COOKIE))
            .then(|| Principal {
                name: Self::USER.to_string(),
            })
            .ok_or(ApiError::Unauthorized)
    }
}
