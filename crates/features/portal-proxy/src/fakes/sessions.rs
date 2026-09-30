use axum::http::HeaderMap;
use axum::http::header::COOKIE;
use portal_feature::{ApiError, Gate, Principal, Rights};

pub struct Sessions;

impl Sessions {
    pub const COOKIE: &'static str = "home_portal_session=anna-token";
    pub const USER: &'static str = "anna";
    pub const GUEST_COOKIE: &'static str = "home_portal_session=guest-token";
}

impl Gate for Sessions {
    fn admit(&self, headers: &HeaderMap) -> Result<Principal, ApiError> {
        let sent = |cookie: &str| {
            headers
                .get_all(COOKIE)
                .iter()
                .filter_map(|value| value.to_str().ok())
                .any(|value| value.contains(cookie))
        };
        if sent(Self::GUEST_COOKIE) {
            return Ok(Principal::member("guest", None, Rights::none()));
        }
        sent(Self::COOKIE)
            .then(|| Principal::admin(Self::USER))
            .ok_or(ApiError::Unauthorized)
    }
}
